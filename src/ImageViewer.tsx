import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'

import type { Project } from './ProjectHistory'

export function ImageViewer({ project }: { project: Project }) {
  const viewport = useRef<HTMLDivElement>(null)
  const drag = useRef<{ x: number; y: number; left: number; top: number } | null>(null)
  const [url, setUrl] = useState('')
  const [error, setError] = useState('')
  const [fit, setFit] = useState(1)
  const [zoom, setZoom] = useState<number | null>(null)
  const scale = zoom ?? fit
  useEffect(() => {
    let active = true
    let objectUrl = ''
    invoke<ArrayBuffer>('get_project_png', { id: project.id })
      .then((bytes) => {
        if (!active) return
        objectUrl = URL.createObjectURL(new Blob([bytes], { type: 'image/png' }))
        setUrl(objectUrl)
      })
      .catch((error: unknown) => {
        if (active) setError(String(error))
      })
    return () => {
      active = false
      if (objectUrl) URL.revokeObjectURL(objectUrl)
    }
  }, [project.id])
  useEffect(() => {
    const element = viewport.current
    if (!element) return
    const observer = new ResizeObserver(() => {
      setFit(
        Math.min(
          (element.clientWidth - 24) / project.width,
          (element.clientHeight - 24) / project.height,
          1,
        ),
      )
    })
    observer.observe(element)
    return () => observer.disconnect()
  }, [project.width, project.height])
  function changeZoom(next: number | null) {
    setZoom(next === null ? null : Math.min(16, Math.max(0.01, next)))
  }
  return (
    <>
      <div className="window-links">
        <nav aria-label="图片缩放">
          <button onClick={() => changeZoom(null)} aria-pressed={zoom === null}>
            适应窗口
          </button>
          <button onClick={() => changeZoom(1 / window.devicePixelRatio)}>实际像素</button>
          <button onClick={() => changeZoom(scale / 1.25)} aria-label="缩小图片">
            −
          </button>
          <output aria-label="缩放比例">
            {Math.round(scale * window.devicePixelRatio * 100)}%
          </output>
          <button onClick={() => changeZoom(scale * 1.25)} aria-label="放大图片">
            ＋
          </button>
        </nav>
      </div>
      {error && <p role="alert">图片加载失败：{error}</p>}
      {!url && !error && <p role="status">正在加载原图…</p>}
      <div
        ref={viewport}
        className="image-viewport"
        tabIndex={0}
        role="region"
        aria-label="图片画布，可拖动或使用方向键平移"
        onKeyDown={(event) => {
          const directions: Record<string, [number, number]> = {
            ArrowLeft: [-80, 0],
            ArrowRight: [80, 0],
            ArrowUp: [0, -80],
            ArrowDown: [0, 80],
          }
          const direction = directions[event.key]
          if (direction) {
            event.preventDefault()
            event.currentTarget.scrollBy(...direction)
          }
        }}
        onPointerDown={(event) => {
          if (event.button !== 0) return
          const element = event.currentTarget
          drag.current = {
            x: event.clientX,
            y: event.clientY,
            left: element.scrollLeft,
            top: element.scrollTop,
          }
          element.setPointerCapture(event.pointerId)
        }}
        onPointerMove={(event) => {
          if (!drag.current) return
          event.currentTarget.scrollLeft = drag.current.left + drag.current.x - event.clientX
          event.currentTarget.scrollTop = drag.current.top + drag.current.y - event.clientY
        }}
        onPointerUp={() => {
          drag.current = null
        }}
        onPointerCancel={() => {
          drag.current = null
        }}
        onLostPointerCapture={() => {
          drag.current = null
        }}
      >
        <div
          className="image-stage"
          style={{ width: project.width * scale + 24, height: project.height * scale + 24 }}
        >
          {url && (
            <img
              src={url}
              alt={project.name}
              draggable={false}
              style={{ width: project.width * scale, height: project.height * scale }}
              onError={() => setError('无法显示图片，请重新打开项目。')}
            />
          )}
        </div>
      </div>
      <p className="muted">拖动或使用滚动条平移。实际像素将一个图像像素对应到一个屏幕像素。</p>
    </>
  )
}
