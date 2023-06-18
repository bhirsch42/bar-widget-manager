import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";
import { invoke, dialog } from "@tauri-apps/api";

// dialog
//   .open({
//     directory: true,
//   })
//   .then(console.log);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
