import React, { useEffect } from "react";
import { events } from "@/types/bindings";
import { useSetAtom } from "jotai";
import {
  appState as appStateAtom,
  sessionCode as sessionCodeAtom,
} from "@/atoms/app";

const GlobalListener: React.FC = () => {
  const setAppState = useSetAtom(appStateAtom);
  const setSessionCode = useSetAtom(sessionCodeAtom);

  useEffect(() => {
    const serverFailEventPromise = events.serverFailEvent.listen((data) => {
      console.error("serverFailEvent", data.payload);
    });

    const serverStartedEventPromise = events.serverStartedEvent.listen(
      (data) => {
        setSessionCode(data.payload);
        setAppState("hosting");
      },
    );

    return () => {
      serverFailEventPromise.then((unlisten) => unlisten());
      serverStartedEventPromise.then((unlisten) => unlisten());
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
