import { invoke, isTauri } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

interface Actions {
  editor: boolean
  copy: boolean
  save: boolean
}

export function CaptureActions() {
  const [actions, setActions] = useState<Actions | null>(null)
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState('')
  const [error, setError] = useState('')
  useEffect(() => {
    if (!isTauri()) return
    let active = true
    invoke<Actions>('get_capture_actions')
      .then((value) => {
        if (active) setActions(value)
      })
      .catch((error: unknown) => {
        if (active) setError(String(error))
      })
    return () => {
      active = false
    }
  }, [])
  async function save() {
    setBusy(true)
    setError('')
    setMessage('')
    try {
      await invoke('set_capture_actions', { actions })
      setMessage('截图后动作已保存。')
    } catch (error) {
      setError(String(error))
    } finally {
      setBusy(false)
    }
  }
  return (
    <section className="workspace-card" aria-labelledby="capture-actions-heading">
      <h2 id="capture-actions-heading">截图完成后</h2>
      <p>可同时启用多个动作，至少保留一项。自动保存的 PNG 位于保存位置下的 exports 文件夹。</p>
      {!isTauri() && <p>请在桌面应用中设置。</p>}
      {actions && (
        <>
          {(['editor', 'copy', 'save'] as const).map((key) => (
            <label className="action-choice" key={key}>
              <input
                type="checkbox"
                checked={actions[key]}
                disabled={busy}
                onChange={(event) => {
                  setActions({ ...actions, [key]: event.target.checked })
                  setMessage('')
                }}
              />
              {{ editor: '打开编辑器', copy: '复制到剪贴板', save: '自动保存 PNG' }[key]}
            </label>
          ))}
          <button
            className="primary-action"
            disabled={busy || !Object.values(actions).some(Boolean)}
            onClick={save}
          >
            保存动作设置
          </button>
          {!Object.values(actions).some(Boolean) && <p role="status">请至少选择一个动作。</p>}
        </>
      )}
      {message && <p role="status">{message}</p>}
      {error && <p role="alert">{error}</p>}
    </section>
  )
}

export function ProjectDelivery({ id }: { id: string }) {
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState('')
  const [error, setError] = useState('')
  async function deliver(save: boolean) {
    setBusy(true)
    setMessage('')
    setError('')
    try {
      if (save) {
        const path = await invoke<string | null>('save_project', { id })
        setMessage(path ? `已保存：${path}` : '已取消保存。')
      } else {
        await invoke('copy_project', { id })
        setMessage('已复制图片。')
      }
    } catch (error) {
      setError(String(error))
    } finally {
      setBusy(false)
    }
  }
  return (
    <>
      <div className="window-links">
        <nav aria-label="图片交付">
          <button disabled={busy} onClick={() => deliver(false)}>
            复制图片
          </button>
          <button disabled={busy} onClick={() => deliver(true)}>
            另存为 PNG…
          </button>
        </nav>
      </div>
      {message && (
        <p className="storage-path" role="status">
          {message}
        </p>
      )}
      {error && <p role="alert">{error}</p>}
    </>
  )
}
