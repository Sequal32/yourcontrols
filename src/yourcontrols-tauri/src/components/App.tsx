import React, { useEffect } from "react";
import Settings from "@/components/Settings";
import Join from "@/components/Join";
import Host from "@/components/Host";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useAtomValue } from "jotai";
import { appState as appStateAtom } from "@/atoms/app";
import Server from "@/components/Server";

const DefaultPage: React.FC = React.memo(() => (
  <>
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
  </>
));

const App: React.FC = () => {
  const appState = useAtomValue(appStateAtom);

  // todo: move to theme provider and add theme option
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

  // todo: maybe use routes to persist even after reload?
  const switchAppState = (): React.ReactNode => {
    switch (appState) {
      case "hosting":
        return <Server />;
      case "connected":
        return "Connected";
      default:
        return <DefaultPage />;
    }
  };

  return (
    <div
      className="flex h-screen w-screen select-none justify-center overflow-hidden overflow-y-auto p-2"
      onContextMenu={disableContextMenuInProduction}
    >
      <div className="flex w-full max-w-screen-lg flex-col">
        {switchAppState()}
      </div>
    </div>
  );
};

export default App;
