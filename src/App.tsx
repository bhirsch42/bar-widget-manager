import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";
import { invoke } from "@tauri-apps/api";

type Widget = {
  fileName: string;
  text: string;
  isInstalled: string;
};

function useWidgetList(): { widgets: Widget[]; isLoading: boolean } {
  const [widgets, setWidgets] = useState<Widget[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  useEffect(() => {
    invoke("get_all_widgets").then((response) => {
      const responseWidgets = response as Widget[];
      console.log("AAAAH", responseWidgets);
      setWidgets(responseWidgets);
    });
  });

  return { widgets, isLoading };
}

function App() {
  const { widgets } = useWidgetList();

  return (
    <div>
      {widgets.map((widget) => (
        <div key={widget.fileName}>
          <div>{widget.fileName}</div>
        </div>
      ))}
    </div>
  );
}

export default App;
