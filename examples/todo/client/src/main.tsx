import { createInertiaApp } from "@inertiajs/react";
import type { ComponentType } from "react";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

const pages = import.meta.glob<{ default: ComponentType }>("./pages/**/*.tsx", {
  eager: true,
});

createInertiaApp({
  resolve: (name) => pages[`./pages/${name}.tsx`],
  setup({ el, App, props }) {
    createRoot(el).render(
      <StrictMode>
        <App {...props} />
      </StrictMode>,
    );
  },
  progress: {
    color: "#e86f51",
    showSpinner: false,
  },
});
