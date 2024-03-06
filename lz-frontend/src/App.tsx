import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MyBookmarks } from "./MyBookmarks.tsx";

function App() {
  const queryClient = new QueryClient();

  return (
    <>
      <QueryClientProvider client={queryClient}>
        <MyBookmarks />
      </QueryClientProvider>
    </>
  );
}

export default App;
