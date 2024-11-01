import React, { useEffect } from "react";
import Settings from "@/components/Settings";
import Join from "@/components/Join";
import Host from "@/components/Host";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";

const App: React.FC = () => {
  useEffect(() => {
    const root = window.document.documentElement;

    root.classList.remove("light", "dark");

    const systemTheme = window.matchMedia("(prefers-color-scheme: dark)")
      .matches
      ? "dark"
      : "light";

    root.classList.add(systemTheme);
  }, []);

  return (
    <div className="flex h-screen w-screen select-none flex-col p-2">
      <Settings />
      <Tabs defaultValue="join" className="mt-2 h-full">
        <TabsList className="grid grid-cols-2">
          <TabsTrigger value="join">Join</TabsTrigger>
          <TabsTrigger value="host">Host</TabsTrigger>
        </TabsList>
        <TabsContent value="join">
          <Join />
        </TabsContent>
        <TabsContent value="host">
          <Host />
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default App;
