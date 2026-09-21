import { invoke, isTauri } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

interface Project {
  version: number
  id: string
  name: string
  created_at: string
  width: number
  height: number
  source_path: string | null
}
interface Library {
  projects: Project[]
  warnings: string[]
}
interface ImportReport {
  imported: number
  skipped: number
  warnings: string[]
}

export function ProjectHistory() {
  const [library, setLibrary] = useState<Library>({ projects: [], warnings: [] })
  const [selected, setSelected] = useState<Project | null>(null)
  const [busy, setBusy] = useState(isTauri())
  const [error, setError] = useState('')
  const [message, setMessage] = useState('')
  useEffect(() => {
    if (!isTauri()) return
    let active = true
    invoke<Library>('list_projects')
      .then((result) => {
        if (active) setLibrary(result)
      })
      .catch((error: unknown) => {
        if (active) setError(String(error))
      })
      .finally(() => {
        if (active) setBusy(false)
      })
    return () => {
      active = false
    }
  }, [])
  async function refresh(importLegacy: boolean) {
    setBusy(true)
    setError('')
    setMessage('')
    setSelected(null)
    try {
      const report = importLegacy ? await invoke<ImportReport>('import_legacy_projects') : null
      const result = await invoke<Library>('list_projects')
      setLibrary({
        projects: result.projects,
        warnings: [...new Set([...result.warnings, ...(report?.warnings ?? [])])],
      })
      if (report)
        setMessage(
          `已导入 ${report.imported} 项，跳过 ${report.skipped} 项重复图片。原文件未移动或修改。`,
        )
    } catch (error) {
      setError(String(error))
    } finally {
      setBusy(false)
    }
  }
  async function inspect(id: string) {
    setBusy(true)
    setError('')
    setSelected(null)
    try {
      setSelected(await invoke<Project>('get_project', { id }))
    } catch (error) {
      setError(String(error))
    } finally {
      setBusy(false)
    }
  }
  if (!isTauri())
    return (
      <section className="workspace-card">
        <p>请在桌面应用中查看本机项目。</p>
      </section>
    )
  return (
    <section className="workspace-card" aria-labelledby="projects-heading" aria-busy={busy}>
      <h2 id="projects-heading">已保存项目</h2>
      <div className="window-links">
        <nav aria-label="项目管理">
          <button disabled={busy} onClick={() => refresh(false)}>
            刷新列表
          </button>
          <button disabled={busy} onClick={() => refresh(true)}>
            导入旧截图目录
          </button>
        </nav>
      </div>
      <p className="muted">导入当前保存位置的 images 文件夹，复制原图，不移动或修改旧文件。</p>
      {busy && <p role="status">正在读取或保存项目…</p>}
      {error && <p role="alert">{error}</p>}
      {message && <p role="status">{message}</p>}
      {!busy && !error && library.projects.length === 0 && (
        <p>暂无已保存项目。可以截图或导入旧截图。</p>
      )}
      <ul className="project-list">
        {library.projects.map((project) => (
          <li key={project.id}>
            <button disabled={busy} onClick={() => inspect(project.id)}>
              <strong>{project.name}</strong>
              <span>
                {project.width} × {project.height} · {new Date(project.created_at).toLocaleString()}
              </span>
            </button>
          </li>
        ))}
      </ul>
      {selected && (
        <div className="capture-result" role="status">
          <h3>{selected.name}</h3>
          <p>
            {selected.width} × {selected.height} 像素 ·{' '}
            {new Date(selected.created_at).toLocaleString()}
          </p>
          <p className="storage-path">
            {selected.source_path ? `导入来源：${selected.source_path}` : '来源：DYD 截图'}
          </p>
          <p className="muted">原图已独立保存。图片查看功能尚未接通。</p>
        </div>
      )}
      {library.warnings.length > 0 && (
        <div role="alert">
          <h3>需要检查的文件</h3>
          <ul>
            {library.warnings.map((warning) => (
              <li className="storage-path" key={warning}>
                {warning}
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  )
}
