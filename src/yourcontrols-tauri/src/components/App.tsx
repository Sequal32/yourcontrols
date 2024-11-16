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

  const disableContextMenuInProduction = (
    e: React.MouseEvent<HTMLDivElement>,
  ) => {
    // Check if dev environment or target is an input field
    if (import.meta.env.DEV || e.target instanceof HTMLInputElement) {
      return;
    }

    e.preventDefault();
  };

  return (
    <div
      className="flex h-screen w-screen select-none justify-center p-2"
      onContextMenu={disableContextMenuInProduction}
    >
      <div className="flex w-full max-w-screen-lg flex-col">
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
    </div>
  );
};

export default App;
