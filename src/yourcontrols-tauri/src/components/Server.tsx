import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { commands, events, MetricsEvent } from "@/types/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import {
  appState as appStateAtom,
  sessionCode as sessionCodeState,
} from "@/atoms/app";
import { useEffect, useRef, useState } from "react";
import { Input } from "@/components/ui/input";
import { Copy } from "lucide-react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useToast } from "@/hooks/use-toast";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";

const Server: React.FC = () => {
  const sessionCodeInputRef = useRef<HTMLInputElement>(null);

  const { toast } = useToast();

  const setAppState = useSetAtom(appStateAtom);
  const sessionCode = useAtomValue(sessionCodeState);

  const [metrics, setMetrics] = useState<MetricsEvent>();

  useEffect(() => {
    const metricsEventPromise = events.metricsEvent.listen((data) => {
      setMetrics(data.payload);
    });

    return () => {
      metricsEventPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleStopServer = () => {
    commands.disconnect().then(() => {
      setAppState("default");
    });
  };

  const handleClickOnSessionCode = () => {
    sessionCodeInputRef.current?.select();
  };

  const handleCopySessionCode = () => {
    writeText(sessionCode ?? "")
      .then(() => {
        toast({
          duration: 5000,
          variant: "constructive",
          title: "Copied session code to clipboard!",
        });
      })
      .catch(() => {
        toast({
          duration: 5000,
          variant: "destructive",
          title: "Could not copy session code to clipboard!",
        });
      });
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Server</CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        <div className="flex h-20 items-center justify-between space-y-0 rounded-lg border p-4">
          <div className="text-base">Session Code</div>
          <div className="flex">
            <Input
              ref={sessionCodeInputRef}
              className="w-[6.25em] font-mono"
              value={sessionCode}
              onFocus={handleClickOnSessionCode}
              // todo: add streamer mode and get the value
              // type={streamerMode ? "password": "text"}
              readOnly
            />
            <Button
              className="ml-2"
              variant="ghost"
              size="icon"
              onClick={handleCopySessionCode}
            >
              <Copy />
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div className="flex h-20 items-center justify-between space-y-0 rounded-lg border p-4">
            <div className="text-base">Ping</div>
            <div className="font-mono">
              {metrics ? (
                <>
                  {metrics.ping.toFixed(0)}
                  <span className="text-sm text-muted-foreground"> ms</span>
                </>
              ) : (
                <Skeleton className="h-[24px] w-[40px]" />
              )}
            </div>
          </div>

          <div className="flex h-20 items-center justify-between space-y-0 rounded-lg border p-4">
            <div className="text-base">Packet Loss</div>
            <div className="font-mono">
              {metrics ? (
                <>
                  {(metrics.packetLoss * 100).toFixed(2)}
                  <span className="text-sm text-muted-foreground"> %</span>
                </>
              ) : (
                <Skeleton className="h-[24px] w-[50px]" />
              )}
            </div>
          </div>

          <div className="flex h-20 items-center justify-between space-y-0 rounded-lg border p-4">
            <div className="text-base">Bandwidth</div>
            <div className="font-mono">
              <div>
                {metrics ? (
                  <>
                    ↓ {metrics.receivedBandwidth.toFixed(2)}
                    <span className="text-sm text-muted-foreground"> KB/s</span>
                  </>
                ) : (
                  <Skeleton className="h-[23px] w-[90px]" />
                )}
              </div>
              <div>
                {metrics ? (
                  <>
                    ↑ {metrics.sentBandwidth.toFixed(2)}
                    <span className="text-sm text-muted-foreground"> KB/s</span>
                  </>
                ) : (
                  <Skeleton className="mt-0.5 h-[23px] w-[90px]" />
                )}
              </div>
            </div>
          </div>

          <div className="flex h-20 items-center justify-between space-y-0 rounded-lg border p-4">
            <div className="text-base">Packets</div>
            <div className="font-mono">
              <div>
                {metrics ? (
                  <>
                    ↓ {metrics.receivedPackets}
                    <span className="text-sm text-muted-foreground">
                      {" "}
                      Packets/s
                    </span>
                  </>
                ) : (
                  <Skeleton className="h-[23px] w-[105px]" />
                )}
              </div>
              <div>
                {metrics ? (
                  <>
                    ↑ {metrics.sentPackets}
                    <span className="text-sm text-muted-foreground">
                      {" "}
                      Packets/s
                    </span>
                  </>
                ) : (
                  <Skeleton className="mt-0.5 h-[23px] w-[105px]" />
                )}
              </div>
            </div>
          </div>
        </div>

        <div className="flex items-center justify-between space-y-0 rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead colSpan={2}>Clients</TableHead>
                {/* <TableHead className="text-right">Status</TableHead> */}
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell className="font-medium">username</TableCell>
                <TableCell className="text-right">In Control</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </CardContent>
      <CardFooter className="flex w-full justify-center">
        <Button
          variant="destructive"
          className="w-full max-w-3xl"
          onClick={handleStopServer}
        >
          Stop Server
        </Button>
      </CardFooter>
    </Card>
  );
};

export default Server;
