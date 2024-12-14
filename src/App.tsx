import { Excalidraw, MainMenu } from '@excalidraw/excalidraw'
import './App.css'

function App() {
  const renderMenu = () => {
    return (
      <MainMenu>
        <MainMenu.DefaultItems.LoadScene />
        <MainMenu.DefaultItems.SaveToActiveFile />
        {/* FIXME we should to test for this inside the item itself */}
        <MainMenu.DefaultItems.Export />
        {/* FIXME we should to test for this inside the item itself */}
        <MainMenu.DefaultItems.SaveAsImage />
        {/* <MainMenu.DefaultItems.SearchMenu /> */}
        <MainMenu.DefaultItems.Help />
        <MainMenu.DefaultItems.ClearCanvas />
        <MainMenu.Separator />
        <MainMenu.DefaultItems.ToggleTheme />
        <MainMenu.DefaultItems.ChangeCanvasBackground />
      </MainMenu>
    )
  }

  return (
    <div className="container">
      <Excalidraw
        initialData={{
          appState: { viewBackgroundColor: '#FFFFFF00' },
        }}
        renderTopRightUI={() => {
          return (
            <button
              style={{
                background: '#70b1ec',
                border: 'none',
                color: '#fff',
                width: 'max-content',
                fontWeight: 'bold',
              }}
              onClick={() => window.alert('This is dummy top right UI')}
            >
              Click me
            </button>
          )
        }}
      >
        {renderMenu()}
      </Excalidraw>
    </div>
  )
}

export default App
