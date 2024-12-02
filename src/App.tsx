import { useEffect } from "react";
import "./App.css";
import { Excalidraw } from "@excalidraw/excalidraw";

function App() {
  useEffect(() => {
    console.log(document.documentElement.style);
  }, []);
  return (
    <div className="container">
      <Excalidraw />
    </div>
  );
}

export default App;
