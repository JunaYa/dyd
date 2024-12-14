// excalidraw file list
import { readDir } from '@tauri-apps/plugin-fs'
import { LazyStore } from '@tauri-apps/plugin-store'
import { useEffect, useState } from 'react'

interface ExcalidrawFile {
  id: string
  url: string
  name: string
  size?: number
  createdAt?: string
  modifiedAt?: string
}

function FileList() {
  const store = new LazyStore('settings.json')
  const [list, setList] = useState<ExcalidrawFile[]>([])

  async function loadData() {
    const val = await store.get<{ value: string }>('screenshot_path')

    const entries = await readDir(val?.value ? `${val?.value}/images` : '')

    const tList = entries.map(entry => ({
      id: entry.name,
      url: `${val?.value}/images/${entry.name}`,
      name: entry.name,
      size: 0,
      createdAt: '',
      modifiedAt: '',
    }))
    setList(tList)
  }

  useEffect(() => {
    loadData()
  }, [])

  return (
    <div>
      {list.map(item => (
        <div key={item.id}>
          <div>{item.name}</div>
          <div>{item.size}</div>
          <div>{item.createdAt}</div>
          <div>{item.modifiedAt}</div>
        </div>
      ))}
    </div>
  )
}

export default FileList
