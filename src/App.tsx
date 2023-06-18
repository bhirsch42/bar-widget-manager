import { useEffect, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api";

type Widget = {
  filename: string;
  body: string;
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
  }, []);

  return { widgets, isLoading };
}

function App() {
  const { widgets } = useWidgetList();

  return (
    <div>
      {widgets.map((widget) => (
        <div key={widget.filename}>
          <div>{widget.filename}</div>
          <pre>{widget.body}</pre>
        </div>
      ))}
    </div>
  );
}

export default App;
