import { invoke, isTauri } from '@tauri-apps/api/core'
import { useState } from 'react'

const destinations = [
  ['main', '/', '白板'],
  ['editor', '/editor', '编辑器'],
  ['history', '/history', '历史'],
  ['setting', '/setting', '设置'],
  ['startup', '/startup', '使用指南'],
] as const

export function WindowLinks({ current = 'main' }: { current?: string }) {
  const [error, setError] = useState('')
  const [pending, setPending] = useState(false)
  async function open(name: string) {
    setPending(true)
    setError('')
    try {
      await invoke('open_workspace_window', { name })
    } catch (error) {
      setError(`无法打开窗口：${String(error)}`)
    } finally {
      setPending(false)
    }
  }
  return (
    <div className="window-links">
      <nav aria-label="应用窗口">
        {destinations
          .filter(([name]) => name !== current)
          .map(([name, url, label]) =>
            isTauri() ? (
              <button key={name} disabled={pending} onClick={() => open(name)}>
                {label}
              </button>
            ) : (
              <a key={name} href={url} target="_blank" rel="noreferrer">
                {label}
              </a>
            ),
          )}
      </nav>
      {error && <p role="alert">{error}</p>}
    </div>
  )
}
