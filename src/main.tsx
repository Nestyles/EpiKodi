import React from "react";
import { Provider } from "@/components/ui/provider";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <Provider defaultTheme="dark">
        <App />
      </Provider>
    </BrowserRouter>
  </React.StrictMode>,
);
