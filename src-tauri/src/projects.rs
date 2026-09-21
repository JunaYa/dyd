use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub width: u32,
    pub height: u32,
    pub source_path: Option<String>,
}

#[derive(Default, Serialize)]
pub struct Library {
    pub projects: Vec<Project>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct ImportReport {
    pub imported: usize,
    pub skipped: usize,
    pub warnings: Vec<String>,
}

pub struct ProjectStore {
    root: PathBuf,
    _lock: File,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn regular_file(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        bail!("不是普通文件：{}", path.display());
    }
    Ok(())
}

fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    Ok(())
}

fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let temporary = path.with_extension("json.tmp");
    // The store lock serializes writers; a leftover temporary file is safe to replace.
    if temporary.try_exists()? {
        regular_file(&temporary)?;
        fs::remove_file(&temporary)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    sync_dir(path.parent().context("Missing parent directory")?)
}

fn same_file_contents(a: &Path, b: &Path) -> Result<bool> {
    if fs::metadata(a)?.len() != fs::metadata(b)?.len() {
        return Ok(false);
    }
    let (mut a, mut b) = (File::open(a)?, File::open(b)?);
    let (mut left, mut right) = ([0u8; 65536], [0u8; 65536]);
    loop {
        let count = a.read(&mut left)?;
        if count == 0 {
            return Ok(true);
        }
        b.read_exact(&mut right[..count])?;
        if left[..count] != right[..count] {
            return Ok(false);
        }
    }
}

impl ProjectStore {
    pub fn open(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)?;
        if root.join(".store.lock").try_exists()? {
            regular_file(&root.join(".store.lock"))?;
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join(".store.lock"))?;
        lock.lock()?;
        Ok(Self { root, _lock: lock })
    }

    fn read_directory(&self, directory: &Path, id: &str) -> Result<Project> {
        if !valid_id(id) || !fs::symlink_metadata(directory)?.file_type().is_dir() {
            bail!("无效项目目录");
        }
        for name in ["project.json", "original.bin", "thumbnail.png"] {
            regular_file(&directory.join(name))?;
        }
        if fs::metadata(directory.join("project.json"))?.len() > 65536 {
            bail!("项目元数据过大");
        }
        let project: Project = serde_json::from_slice(&fs::read(directory.join("project.json"))?)?;
        if project.version != 1 || project.id != id || project.width == 0 || project.height == 0 {
            bail!("不支持或无效的项目元数据");
        }
        chrono::DateTime::parse_from_rfc3339(&project.created_at).context("无效的项目创建时间")?;
        let original = image::ImageReader::open(directory.join("original.bin"))?
            .with_guessed_format()?
            .into_dimensions()?;
        if original != (project.width, project.height) {
            bail!("原图尺寸与元数据不一致");
        }
        let (width, height) = image::image_dimensions(directory.join("thumbnail.png"))?;
        if width == 0 || height == 0 || width > 320 || height > 320 {
            bail!("缩略图尺寸无效");
        }
        Ok(project)
    }

    pub fn png(&self, id: &str) -> Result<Vec<u8>> {
        self.get(id)?;
        let path = self.root.join(id).join("original.bin");
        if fs::metadata(&path)?.len() > 128 * 1024 * 1024 {
            bail!("图片文件超过 128 MiB 限制");
        }
        let mut reader = image::ImageReader::open(path)?.with_guessed_format()?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(16384);
        limits.max_image_height = Some(16384);
        limits.max_alloc = Some(128 * 1024 * 1024);
        reader.limits(limits);
        crate::common::encode_png(&reader.decode()?).map_err(anyhow::Error::msg)
    }

    pub fn get(&self, id: &str) -> Result<Project> {
        if !valid_id(id) {
            bail!("无效项目 ID");
        }
        self.read_directory(&self.root.join(id), id)
    }

    pub fn list(&self) -> Result<Library> {
        let mut library = Library::default();
        let entries: Vec<_> = fs::read_dir(&self.root)?.collect::<std::io::Result<_>>()?;
        for entry in &entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(id) = name.strip_prefix(".pending-") {
                match self.read_directory(&entry.path(), id) {
                    Ok(_) if !self.root.join(id).try_exists()? => {
                        fs::rename(entry.path(), self.root.join(id))?;
                        sync_dir(&self.root)?;
                    }
                    Ok(_) => library
                        .warnings
                        .push(format!("保留冲突的未提交项目：{name}")),
                    Err(error) => library
                        .warnings
                        .push(format!("保留未完成项目 {name}：{error}")),
                }
            }
        }
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || matches!(name.as_str(), "index.json" | "index.json.tmp") {
                continue;
            }
            match self.get(&name) {
                Ok(project) => library.projects.push(project),
                Err(error) => library
                    .warnings
                    .push(format!("项目 {name} 无法读取：{error}")),
            }
        }
        library.projects.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        if let Err(error) = atomic_json(
            &self.root.join("index.json"),
            &serde_json::json!({ "version": 1, "projects": library.projects }),
        ) {
            library
                .warnings
                .push(format!("索引更新失败，可从项目目录重建：{error}"));
        }
        Ok(library)
    }

    pub fn create(&self, source: &Path, legacy: bool) -> Result<Project> {
        regular_file(source)?;
        if fs::metadata(source)?.len() > 128 * 1024 * 1024 {
            bail!("图片文件超过 128 MiB 限制");
        }
        let id = format!(
            "{}-{}-{}",
            chrono::Utc::now()
                .timestamp_nanos_opt()
                .context("无效时钟")?,
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let staging = self.root.join(format!(".pending-{id}"));
        fs::create_dir(&staging)?;
        let result = (|| -> Result<Project> {
            let original = staging.join("original.bin");
            fs::copy(source, &original)?;
            File::open(&original)?.sync_all()?;
            let mut reader = image::ImageReader::open(&original)?.with_guessed_format()?;
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(16384);
            limits.max_image_height = Some(16384);
            limits.max_alloc = Some(128 * 1024 * 1024);
            reader.limits(limits);
            let image = reader.decode().context("图片无法解码或超过内存/尺寸限制")?;
            let thumbnail = staging.join("thumbnail.png");
            image.thumbnail(320, 320).save(&thumbnail)?;
            File::open(&thumbnail)?.sync_all()?;
            let timestamp = if legacy {
                fs::metadata(source)?.modified()?
            } else {
                std::time::SystemTime::now()
            };
            let project = Project {
                version: 1,
                id: id.clone(),
                name: source
                    .file_name()
                    .context("Missing filename")?
                    .to_string_lossy()
                    .into_owned(),
                created_at: chrono::DateTime::<chrono::Utc>::from(timestamp)
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                width: image.width(),
                height: image.height(),
                source_path: if legacy {
                    Some(source.canonicalize()?.to_string_lossy().into_owned())
                } else {
                    None
                },
            };
            atomic_json(&staging.join("project.json"), &project)?;
            let destination = self.root.join(&id);
            if destination.try_exists()? {
                bail!("项目 ID 冲突");
            }
            fs::rename(&staging, &destination)?;
            sync_dir(&self.root)?;
            Ok(project)
        })();
        if result.is_err() && staging.exists() {
            fs::remove_dir_all(&staging).context("清理本次未提交项目失败")?;
        }
        result
    }

    pub fn import_legacy(&self, directory: &Path) -> Result<ImportReport> {
        let mut library = self.list()?;
        let mut report = ImportReport {
            imported: 0,
            skipped: 0,
            warnings: std::mem::take(&mut library.warnings),
        };
        if !directory.try_exists()? {
            return Ok(report);
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let source = entry.path();
            let extension = source
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif"]
                .contains(&extension.as_str())
            {
                continue;
            }
            let result = (|| -> Result<Option<Project>> {
                let canonical = source.canonicalize()?.to_string_lossy().into_owned();
                for project in &library.projects {
                    if project.source_path.as_deref() == Some(&canonical)
                        && same_file_contents(
                            &source,
                            &self.root.join(&project.id).join("original.bin"),
                        )?
                    {
                        return Ok(None);
                    }
                    // New captures keep a compatibility copy in images; do not import it twice.
                    if project.source_path.is_none()
                        && project.name == entry.file_name().to_string_lossy()
                        && same_file_contents(
                            &source,
                            &self.root.join(&project.id).join("original.bin"),
                        )?
                    {
                        return Ok(None);
                    }
                }
                Ok(Some(self.create(&source, true)?))
            })();
            match result {
                Ok(Some(project)) => {
                    report.imported += 1;
                    library.projects.push(project);
                }
                Ok(None) => report.skipped += 1,
                Err(error) => report
                    .warnings
                    .push(format!("{}：{error:#}", source.display())),
            }
        }
        report.warnings.extend(self.list()?.warnings);
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "dyd-project-test-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn image(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            image::RgbaImage::from_pixel(640, 400, image::Rgba([20, 40, 60, 128]))
                .save(&path)
                .unwrap();
            path
        }
        fn store(&self) -> ProjectStore {
            ProjectStore::open(self.0.join("projects")).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn project_survives_restart_and_preserves_original_bytes() {
        let fixture = Fixture::new();
        let source = fixture.image("source.png");
        let before = fs::read(&source).unwrap();
        let project = fixture.store().create(&source, false).unwrap();
        let store = fixture.store();
        let loaded = store.get(&project.id).unwrap();
        assert_eq!((loaded.width, loaded.height), (640, 400));
        assert_eq!(loaded.version, 1);
        assert_eq!(fs::read(&source).unwrap(), before);
        assert_eq!(
            fs::read(store.root.join(&project.id).join("original.bin")).unwrap(),
            before
        );
        assert_eq!(
            image::image_dimensions(store.root.join(&project.id).join("thumbnail.png")).unwrap(),
            (320, 200)
        );
        assert_eq!(store.list().unwrap().projects.len(), 1);
    }
    #[test]
    fn index_is_rebuilt_after_missing_or_corrupt_index() {
        let fixture = Fixture::new();
        let source = fixture.image("source.png");
        let store = fixture.store();
        store.create(&source, false).unwrap();
        assert_eq!(store.list().unwrap().projects.len(), 1);
        fs::write(store.root.join("index.json"), b"broken").unwrap();
        assert_eq!(store.list().unwrap().projects.len(), 1);
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(store.root.join("index.json")).unwrap()).unwrap();
        assert_eq!(index["projects"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn complete_staging_recovers_but_incomplete_staging_is_only_reported() {
        let fixture = Fixture::new();
        let source = fixture.image("source.png");
        let store = fixture.store();
        let project = store.create(&source, false).unwrap();
        fs::rename(
            store.root.join(&project.id),
            store.root.join(format!(".pending-{}", project.id)),
        )
        .unwrap();
        let incomplete = store.root.join(".pending-interrupted");
        fs::create_dir(&incomplete).unwrap();
        fs::write(incomplete.join("original.bin"), b"partial").unwrap();
        let library = store.list().unwrap();
        assert_eq!(library.projects.len(), 1);
        assert_eq!(library.warnings.len(), 1);
        assert!(incomplete.join("original.bin").exists());
    }
    #[test]
    fn legacy_import_is_read_only_repeatable_and_tolerates_bad_files() {
        let fixture = Fixture::new();
        let directory = fixture.0.join("images");
        fs::create_dir(&directory).unwrap();
        let source = fixture.image("images/legacy.png");
        let before = fs::read(&source).unwrap();
        fs::write(directory.join("bad.png"), b"invalid").unwrap();
        let store = fixture.store();
        let first = store.import_legacy(&directory).unwrap();
        assert_eq!(first.imported, 1);
        assert_eq!(first.warnings.len(), 1);
        let second = store.import_legacy(&directory).unwrap();
        assert_eq!(second.imported, 0);
        assert_eq!(second.skipped, 1);
        assert_eq!(fs::read(source).unwrap(), before);
        assert_eq!(fs::read(directory.join("bad.png")).unwrap(), b"invalid");
    }
    #[test]
    fn invalid_images_never_enter_history_and_invalid_ids_are_rejected() {
        let fixture = Fixture::new();
        let source = fixture.0.join("bad.png");
        fs::write(&source, b"bad").unwrap();
        let store = fixture.store();
        assert!(store.create(&source, false).is_err());
        assert!(store.list().unwrap().projects.is_empty());
        for id in ["", "..", "../outside", "/tmp/file", "a/b"] {
            assert!(store.get(id).is_err());
        }
        assert_eq!(fs::read(source).unwrap(), b"bad");
    }
    #[test]
    fn unknown_versions_are_preserved_and_reported() {
        let fixture = Fixture::new();
        let source = fixture.image("source.png");
        let store = fixture.store();
        let mut project = store.create(&source, false).unwrap();
        project.version = 99;
        atomic_json(&store.root.join(&project.id).join("project.json"), &project).unwrap();
        let library = store.list().unwrap();
        assert!(library.projects.is_empty());
        assert_eq!(library.warnings.len(), 1);
        assert!(store.root.join(&project.id).join("original.bin").exists());
    }
    #[test]
    fn current_capture_copy_is_not_imported_twice() {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join("images")).unwrap();
        let source = fixture.image("images/current.png");
        let store = fixture.store();
        store.create(&source, false).unwrap();
        let report = store.import_legacy(&fixture.0.join("images")).unwrap();
        assert_eq!(report.imported, 0);
        assert_eq!(report.skipped, 1);
    }
    #[test]
    fn simultaneous_writers_keep_all_committed_projects() {
        let fixture = Fixture::new();
        let source = fixture.image("source.png");
        let mut workers = Vec::new();
        for _ in 0..4 {
            let root = fixture.0.join("projects");
            let source = source.clone();
            workers.push(std::thread::spawn(move || {
                let store = ProjectStore::open(root).unwrap();
                let project = store.create(&source, false).unwrap();
                store.list().unwrap();
                project.id
            }));
        }
        let ids: std::collections::HashSet<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(ids.len(), 4);
        assert_eq!(fixture.store().list().unwrap().projects.len(), 4);
    }
    #[cfg(unix)]
    #[test]
    fn symlinked_project_assets_and_index_temps_are_not_followed() {
        let fixture = Fixture::new();
        let source = fixture.image("source.png");
        let store = fixture.store();
        let project = store.create(&source, false).unwrap();
        let original = store.root.join(&project.id).join("original.bin");
        fs::remove_file(&original).unwrap();
        std::os::unix::fs::symlink(&source, &original).unwrap();
        assert!(store.get(&project.id).is_err());
        let external = fixture.0.join("external.txt");
        fs::write(&external, b"keep").unwrap();
        std::os::unix::fs::symlink(&external, store.root.join("index.json.tmp")).unwrap();
        assert!(!store.list().unwrap().warnings.is_empty());
        assert_eq!(fs::read(external).unwrap(), b"keep");
    }
}
