import { invoke, isTauri } from '@tauri-apps/api/core'
import { LazyStore } from '@tauri-apps/plugin-store'
import { useEffect, useState } from 'react'

import { CaptureActions } from '../CaptureActions'
import { CapturePanel } from '../CapturePanel'
import { ProjectEditor } from '../ProjectEditor'
import { ProjectHistory } from '../ProjectHistory'
import { WindowLinks } from '../WindowLinks'

const store = new LazyStore('settings.json')
const pages: Record<string, { name: string; title: string; description: string }> = {
  '/setting': { name: 'setting', title: '设置', description: '查看当前保存位置与常用快捷键。' },
  '/startup': {
    name: 'startup',
    title: '欢迎使用 DYD',
    description: '随手画下想法，也可以从托盘开始截图。',
  },
  '/editor': { name: 'editor', title: '图片编辑器', description: '截图与白板使用独立窗口。' },
  '/history': { name: 'history', title: '历史记录', description: '在这里回到之前的截图。' },
  '/preview': { name: 'preview', title: '截图预览', description: '截图查看功能正在接入。' },
}

function Settings() {
  const [path, setPath] = useState<string | null>(null)
  const [error, setError] = useState('')
  useEffect(() => {
    if (!isTauri()) return
    let active = true
    store
      .get<{ value: string }>('screenshot_path')
      .then((saved) => {
        if (!saved || typeof saved.value !== 'string') throw new Error('保存目录配置无效')
        if (active) setPath(saved.value || '应用默认目录')
      })
      .catch((error: unknown) => {
        if (active) setError(String(error))
      })
    return () => {
      active = false
    }
  }, [])
  return (
    <section className="workspace-card" aria-labelledby="storage-heading">
      <h2 id="storage-heading">截图保存位置</h2>
      {!isTauri() ? (
        <p>在桌面应用中查看本机保存位置。</p>
      ) : error ? (
        <p role="alert">读取失败：{error}</p>
      ) : (
        <p className="storage-path" role="status">
          {path ?? '正在读取…'}
        </p>
      )}
      <p className="muted">当前页面仅显示已有配置，目录修改将在后续设置功能中提供。</p>
    </section>
  )
}

function StartDrawing() {
  const [error, setError] = useState('')
  const [pending, setPending] = useState(false)
  async function start() {
    setPending(true)
    setError('')
    try {
      await invoke('finish_startup')
    } catch (error) {
      setError(`无法打开白板：${String(error)}`)
    } finally {
      setPending(false)
    }
  }
  return (
    <>
      {isTauri() ? (
        <button className="primary-action" disabled={pending} onClick={start}>
          开始使用白板
        </button>
      ) : (
        <a className="primary-action" href="/" target="_blank" rel="noreferrer">
          打开白板
        </a>
      )}
      {error && <p role="alert">{error}</p>}
    </>
  )
}

export function Workspace({ page }: { page: string }) {
  const content = pages[page]
  const modifier = /Mac/i.test(navigator.platform) ? '⌘' : 'Ctrl'
  if (!content)
    return (
      <main className="workspace">
        <h1>页面不存在</h1>
        <WindowLinks current="unknown" />
      </main>
    )
  return (
    <main className={`workspace ${content.name === 'preview' ? 'workspace-preview' : ''}`}>
      <header>
        <span className="eyebrow">DYD</span>
        <h1>{content.title}</h1>
        <p>{content.description}</p>
      </header>
      {page === '/setting' && (
        <>
          <Settings />
          <CaptureActions />
        </>
      )}
      {(page === '/setting' || page === '/startup') && (
        <section className="workspace-card" aria-labelledby="shortcuts-heading">
          <h2 id="shortcuts-heading">常用快捷键</h2>
          <dl>
            {[
              ['全屏截图', 'A'],
              ['区域截图', 'S'],
              ['窗口截图', 'W'],
              ['显示白板', 'E'],
            ].map(([label, key]) => (
              <div key={key}>
                <dt>{label}</dt>
                <dd>
                  <kbd>
                    {modifier} + Shift + {key}
                  </kbd>
                </dd>
              </div>
            ))}
          </dl>
          {page === '/startup' && (
            <p className="muted">首次截图时，macOS 可能需要你在系统设置中允许屏幕录制。</p>
          )}
        </section>
      )}
      {page === '/startup' && <StartDrawing />}
      {page === '/editor' && <ProjectEditor />}
      {(page === '/editor' || page === '/preview') && <CapturePanel />}
      {page === '/history' && <ProjectHistory />}
      <WindowLinks current={content.name} />
    </main>
  )
}
