import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";
import { invoke, dialog } from "@tauri-apps/api";

// now we can call our Command!
// Right-click the application background and open the developer tools.
// You will see "Hello, World!" printed in the console!
// invoke("greet", { name: "World" })
//   // `invoke` returns a Promise
//   .then((response) => console.log(response));

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
