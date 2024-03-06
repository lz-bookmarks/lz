import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { LoadBookmarks } from "./LoadBookmarks.tsx";

function App() {
  const queryClient = new QueryClient();

  return (
    <>
      <QueryClientProvider client={queryClient}>
        <LoadBookmarks />
      </QueryClientProvider>
    </>
  );
}

export default App;
