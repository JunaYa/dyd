import { invoke, isTauri } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

import { ImageViewer } from './ImageViewer'
import type { Project } from './ProjectHistory'

interface Selection {
  revision: number
  project: Project | null
}

export function ProjectEditor() {
  const [selection, setSelection] = useState<Selection | null>(null)
  const [error, setError] = useState('')
  useEffect(() => {
    if (!isTauri()) return
    let active = true
    let timer: ReturnType<typeof setTimeout>
    async function read() {
      try {
        const result = await invoke<Selection>('editor_ready')
        if (active) {
          setSelection((previous) => (previous?.revision === result.revision ? previous : result))
          setError('')
        }
      } catch (error) {
        if (active) setError(String(error))
      } finally {
        if (active) timer = setTimeout(read, 250)
      }
    }
    void read()
    return () => {
      active = false
      clearTimeout(timer)
    }
  }, [])
  const project = selection?.project
  return (
    <section className="workspace-card" aria-label="当前图片项目">
      {error && <p role="alert">项目读取失败：{error}</p>}
      {!isTauri() ? (
        <p>请在桌面应用中打开图片项目。</p>
      ) : project ? (
        <div key={selection.revision}>
          <h2>{project.name}</h2>
          <p>
            {project.width} × {project.height} 像素
          </p>
          <ImageViewer project={project} />
        </div>
      ) : (
        <p role="status">{selection ? '截图或从历史记录打开一个项目。' : '正在加载项目…'}</p>
      )}
    </section>
  )
}
