import { Excalidraw, MainMenu } from '@excalidraw/excalidraw'

import '@excalidraw/excalidraw/index.css'
import { WindowLinks } from '../WindowLinks'

function Whiteboard() {
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
        renderTopRightUI={() => <WindowLinks />}
      >
        {renderMenu()}
      </Excalidraw>
    </div>
  )
}

export default Whiteboard
