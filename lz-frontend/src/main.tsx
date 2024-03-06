import React from "react";
import ReactDOM from "react-dom/client";
import "water.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { MyBookmarks } from "./MyBookmarks.tsx";
import { TaggedBookmarks } from "./TaggedBookmarks.tsx";

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<MyBookmarks />} />
          <Route path="/tag/:tag" element={<TaggedBookmarks />} />
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  </React.StrictMode>,
);
