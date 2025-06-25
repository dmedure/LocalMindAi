import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";

// Prevent context menu on production builds
document.addEventListener('contextmenu', e => {
  e.preventDefault();
});

// Prevent text selection for a more native app feel
document.addEventListener('selectstart', e => {
  e.preventDefault();
});

// Disable drag and drop to prevent accidental file drops
document.addEventListener('dragover', e => {
  e.preventDefault();
});

document.addEventListener('drop', e => {
  e.preventDefault();
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);