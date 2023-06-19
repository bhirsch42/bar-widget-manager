import { PropsWithChildren, useEffect, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api";
import clsx from "clsx";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { materialDark } from "react-syntax-highlighter/dist/esm/styles/prism";

type WidgetInfo = {
  name?: string;
  description?: string;
  author?: string;
  date?: string;
  version?: string;
};

type Widget = {
  filename: string;
  body: string;
  info?: WidgetInfo;
};

function useWidgetList(): { widgets: Widget[]; isLoading: boolean } {
  const [widgets, setWidgets] = useState<Widget[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  useEffect(() => {
    invoke("get_all_widgets").then((response) => {
      const responseWidgets = response as Widget[];
      setWidgets(responseWidgets);
    });
  }, []);

  return { widgets, isLoading };
}

function Label({
  children,
  className,
}: PropsWithChildren<{ className?: string }>) {
  return (
    <span className={clsx("font-bold text-sm", className)}>{children}</span>
  );
}

function Widget({ widget }: { widget: Widget }) {
  console.log(materialDark);
  const [showCode, setShowCode] = useState(false);
  return (
    <div
      key={widget.filename}
      className="p-3 bg-zinc-700 rounded-md max-w-full overflow-hidden"
    >
      <div className="text-lg font-bold">
        {widget.info?.name || widget.filename}
      </div>
      <div className="flex gap-3 text-sm">
        {widget.info?.author && (
          <div className="flex items-center">
            <Label className="mr-1">Author</Label> {widget.info?.author}
          </div>
        )}
        {widget.info?.date && (
          <div className="flex items-center">
            <Label className="mr-1">Date</Label> {widget.info?.date}
          </div>
        )}
        {widget.info?.version && (
          <div className="flex items-center">
            <Label className="mr-1">Version</Label> {widget.info?.version}
          </div>
        )}
      </div>
      {widget.info?.description && (
        <div className="mt-2">{widget.info?.description}</div>
      )}
      <button
        onClick={() => setShowCode((value) => !value)}
        className="text-sm underline mt-3"
      >
        {showCode ? "Hide code" : "Show code"}
      </button>

      {showCode && (
        <div className="overflow-auto max-h-96 mt-1 rounded-md">
          <SyntaxHighlighter
            language="lua"
            style={materialDark}
            customStyle={{
              margin: 0,
              borderRadius: ".375rem",
            }}
          >
            {widget.body}
          </SyntaxHighlighter>
        </div>
      )}
    </div>
  );
}

function App() {
  const { widgets } = useWidgetList();

  return (
    <div className="p-3 gap-2 grid grid-flow-row">
      {widgets.map((widget) => (
        <Widget widget={widget} key={widget.filename} />
      ))}
    </div>
  );
}

export default App;
