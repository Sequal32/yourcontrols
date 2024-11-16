import { useEffect } from "react";
import type React from "react";
import { events } from "@/types/bindings";

const GlobalListener: React.FC = () => {
  useEffect(() => {
    const serverFailEventPromise = events.serverFailEvent.listen((data) => {
      console.error("serverFailEvent", data.payload);
    });

    return () => {
      serverFailEventPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    const clientFailEventPromise = events.clientFailEvent.listen((data) => {
      console.error("clientFailEvent", data.payload);
    });

    return () => {
      clientFailEventPromise.then((unlisten) => unlisten());
    };
  }, []);

  return null;
};

export default GlobalListener;
