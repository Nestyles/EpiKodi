import React from "react";
import { Provider } from "@/components/ui/provider";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider defaultTheme="dark">
      <App />
    </Provider>
  </React.StrictMode>,
);
