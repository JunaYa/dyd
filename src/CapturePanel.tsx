import { invoke, isTauri } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

type Kind = 'screen' | 'select' | 'window'
type Task = { id: string; kind: Kind } & (
  | { status: 'capturing' }
  | { status: 'ready'; path: string; project_id: string | null }
  | { status: 'cancelled' }
  | { status: 'failed'; error: string }
)

export function CapturePanel() {
  const [task, setTask] = useState<Task | null>(null)
  const [error, setError] = useState('')
  const [readError, setReadError] = useState('')
  const [permission, setPermission] = useState('')
  const [pending, setPending] = useState(false)
  const [loading, setLoading] = useState(isTauri())
  useEffect(() => {
    if (!isTauri()) return
    let active = true
    let timer: ReturnType<typeof setTimeout>
    async function refresh() {
      try {
        const result = await invoke<Task | null>('get_capture_task')
        if (active) {
          setTask(result)
          setReadError('')
          setLoading(false)
        }
      } catch (error) {
        if (active) {
          setReadError(`无法读取任务：${String(error)}`)
          setLoading(false)
        }
      } finally {
        if (active) timer = setTimeout(refresh, 500)
      }
    }
    void refresh()
    return () => {
      active = false
      clearTimeout(timer)
    }
  }, [])
  async function start(kind: Kind) {
    setPending(true)
    setError('')
    setPermission('')
    try {
      // Polling is the only writer of task state, so a late command response cannot roll it back.
      await invoke<Task>('start_capture', { kind })
    } catch (error) {
      setError(`无法开始截图：${String(error)}`)
    } finally {
      setPending(false)
    }
  }
  async function requestPermission() {
    setPending(true)
    setError('')
    try {
      const allowed = await invoke<boolean>('request_capture_permission')
      setPermission(
        allowed
          ? '屏幕录制权限已允许，可以重试截图。'
          : '请在系统设置中允许 DYD 的屏幕录制权限，然后重启应用。',
      )
    } catch (error) {
      setError(`权限请求失败：${String(error)}`)
    } finally {
      setPending(false)
    }
  }
  return (
    <section className="workspace-card" aria-labelledby="capture-heading">
      <h2 id="capture-heading">截图任务</h2>
      {isTauri() ? (
        <>
          <div className="window-links">
            <nav aria-label="截图方式">
              {(
                [
                  ['screen', '全屏截图'],
                  ['select', '区域截图'],
                  ['window', '窗口截图'],
                ] as const
              ).map(([kind, label]) => (
                <button
                  key={kind}
                  disabled={loading || pending || task?.status === 'capturing'}
                  onClick={() => start(kind)}
                >
                  {label}
                </button>
              ))}
            </nav>
          </div>
          <div className="capture-result" aria-live="polite">
            {loading ? (
              <p>正在读取任务…</p>
            ) : !task ? (
              <p>选择截图方式，或使用托盘与快捷键开始。</p>
            ) : task.status === 'capturing' ? (
              <p>{task.kind === 'screen' ? '正在截图…' : '请选择截图范围，按 Esc 取消。'}</p>
            ) : task.status === 'cancelled' ? (
              <p>已取消截图，未创建项目。</p>
            ) : task.status === 'failed' ? (
              <p role="alert">{task.error}</p>
            ) : (
              <>
                <p>截图已保存</p>
                <p className="storage-path">{task.path}</p>
                {task.project_id && <p>项目已加入历史记录。</p>}
                <p className="muted">图片查看与编辑将在后续版本接入。</p>
              </>
            )}
          </div>
          {task && <p className="muted storage-path">任务 {task.id}</p>}
          {task?.status === 'failed' && (
            <div className="window-links">
              <button disabled={pending} onClick={requestPermission}>
                设置屏幕录制权限
              </button>
            </div>
          )}
          {permission && <p role="status">{permission}</p>}
          {readError && <p role="alert">{readError}</p>}
          {error && <p role="alert">{error}</p>}
        </>
      ) : (
        <p>请在桌面应用中开始截图。</p>
      )}
    </section>
  )
}
