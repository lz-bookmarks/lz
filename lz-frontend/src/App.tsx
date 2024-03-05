import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "./App.css";
import { LoadBookmarks } from "./LoadBookmarks.tsx";

function App() {
  const [reactQueryClient] = useState(
    new QueryClient({
      defaultOptions: {
        queries: {
          networkMode: "offlineFirst", // keep caches as long as possible
          refetchOnWindowFocus: false, // don’t refetch on window focus
          retry: true,
        },
      },
    }),
  );

  return (
    <>
      <QueryClientProvider client={reactQueryClient}>
        <LoadBookmarks />
      </QueryClientProvider>
    </>
  );
}

export default App;
