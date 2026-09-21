import { lazy, Suspense } from 'react'

import { Workspace } from './pages/Workspace'

import './App.css'

const Whiteboard = lazy(() => import('./pages/Whiteboard'))

function App() {
  const page = window.location.pathname.replace(/\/$/, '') || '/'
  return page === '/' ? (
    <Suspense
      fallback={
        <p className="loading-page" role="status">
          正在加载白板…
        </p>
      }
    >
      <Whiteboard />
    </Suspense>
  ) : (
    <Workspace page={page} />
  )
}

export default App
