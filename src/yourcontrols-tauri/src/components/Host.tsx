import React, { useEffect, useState } from "react";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@ui/card";
import { Button } from "@ui/button";
import { ToggleGroup, ToggleGroupItem } from "@ui/toggle-group";
import { Form } from "@ui/form";
import { useForm } from "react-hook-form";
import StyledFormField from "@/components/StyledFormField";
import {
  commands,
  events,
  MetricsEvent,
  ConnectionMethod,
} from "@/types/bindings";
import { useToast } from "@/hooks/use-toast";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";

const formSchema = z.object({
  ipVersion: z
    .string({ required_error: "IP Version is required!" })
    .trim()
    .min(1, { message: "IP Version is required" }),
  hostingMethode: z
    .string({ required_error: "Hosting Mode is required!" })
    .min(1, { message: "Hosting Mode is required" }),
});

// TODO: FormSchema
const Host: React.FC = () => {
  const { toast } = useToast();
  const form = useForm({
    resolver: zodResolver(formSchema),
    reValidateMode: "onSubmit",
    defaultValues: {
      ipVersion: "ipv4",
      hostingMethode: "relay",
    },
  });

  const [publicIp, setPublicIp] = useState<string | null>(null);
  const [metrics, setMetrics] = useState<MetricsEvent>();

  useEffect(() => {
    const metricsEventPromise = events.metricsEvent.listen((data) => {
      setMetrics(data.payload);
    });

    return () => {
      metricsEventPromise.then((unlisten) => unlisten());
    };
  }, []);

  // TODO: store in atom
  useEffect(() => {
    setPublicIp(null);
    const is_ipv6 = form.getValues("ipVersion") === "ipv6";

    commands
      .getPublicIp(is_ipv6)
      .then((ip) => {
        setPublicIp(ip);
      })
      .catch(() => {
        setPublicIp("Could not fetch public IP");
      });
  }, [form.getValues("ipVersion")]);

  // TODO
  const onSubmit = ({
    ipVersion,
    hostingMethode,
  }: z.infer<typeof formSchema>) => {
    commands.startServer(hostingMethode as any).catch((err) => {
      toast({
        duration: 5000,
        variant: "destructive",
        title: "Could not start server!",
        description: err,
      });
    });
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Card>
          <CardHeader>
            <CardTitle>Host</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <StyledFormField
              control={form.control}
              name="ipVersion"
              label="IP Version"
              description={publicIp}
              render={({ field }) => (
                <ToggleGroup
                  type="single"
                  variant="outline"
                  value={field.value}
                  onValueChange={(v) => {
                    if (!v) return;
                    field.onChange(v);
                  }}
                >
                  <ToggleGroupItem value="ipv4">IPv4</ToggleGroupItem>
                  <ToggleGroupItem value="ipv6">IPv6</ToggleGroupItem>
                </ToggleGroup>
              )}
            />

            <StyledFormField
              control={form.control}
              name="hostingMethode"
              label="Hosting Methode"
              render={({ field }) => (
                <ToggleGroup
                  type="single"
                  variant="outline"
                  value={field.value}
                  onValueChange={(v) => {
                    if (!v) return;
                    field.onChange(v);
                  }}
                >
                  <ToggleGroupItem value="relay">Cloud P2P</ToggleGroupItem>
                  <ToggleGroupItem value="cloudServer">
                    Cloud Host
                  </ToggleGroupItem>
                  <ToggleGroupItem value="direct">Direct</ToggleGroupItem>
                </ToggleGroup>
              )}
            />
          </CardContent>
          <CardFooter className="flex w-full justify-center">
            <Button type="submit" className="w-full max-w-3xl">
              Start Server
            </Button>
          </CardFooter>
          {JSON.stringify(metrics)}
        </Card>
      </form>
    </Form>
  );
};

export default Host;
