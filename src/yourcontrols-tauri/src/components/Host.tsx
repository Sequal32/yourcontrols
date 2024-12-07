import React, { useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { Form } from "@/components/ui/form";
import { useForm } from "react-hook-form";
import StyledFormField from "@/components/StyledFormField";
import { commands } from "@/types/bindings";
import { useToast } from "@/hooks/use-toast";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
} from "@/components/ui/accordion";
import { Input } from "@/components/ui/input";

const formSchema = z.object({
  // ToggleGroup needs string values
  isIpv6: z.enum(["false", "true"]),
  hostingMethode: z.enum(["relay", "cloudServer", "direct"]),
  port: z.coerce.number().int("Port needs to be an integer").nullable(),
});

type FormValues = z.infer<typeof formSchema>;

const Host: React.FC = () => {
  const { toast } = useToast();
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    reValidateMode: "onSubmit",
    defaultValues: {
      isIpv6: "false",
      hostingMethode: "relay",
      port: null,
    },
  });

  const [publicIp, setPublicIp] = useState<string | null>(null);

  // TODO: store in atom?
  useEffect(() => {
    commands
      .getPublicIp()
      .then((ip) => {
        setPublicIp(ip);
      })
      .catch(() => {
        setPublicIp("Could not get public IP");
      });
  }, []);

  const onSubmit = ({ hostingMethode, isIpv6, port }: FormValues) => {
    const isIpv6Bool = isIpv6 === "true";

    commands
      .startServer(hostingMethode, isIpv6Bool, port)
      .then(() => {
        // todo: set button to loading, while waiting for server event
      })
      .catch((err) => {
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
              name="isIpv6"
              label="IP Version"
              description={publicIp}
              render={({ field }) => (
                <ToggleGroup
                  type="single"
                  variant="outline"
                  onValueChange={(v) => {
                    if (!v) return;
                    field.onChange(v);
                  }}
                  {...field}
                >
                  <ToggleGroupItem value="false">IPv4</ToggleGroupItem>
                  <ToggleGroupItem value="true">IPv6</ToggleGroupItem>
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
                  onValueChange={(v) => {
                    if (!v) return;
                    field.onChange(v);
                  }}
                  {...field}
                >
                  <ToggleGroupItem value="relay">Cloud P2P</ToggleGroupItem>
                  <ToggleGroupItem value="cloudServer">
                    Cloud Host
                  </ToggleGroupItem>
                  <ToggleGroupItem value="direct">Direct</ToggleGroupItem>
                </ToggleGroup>
              )}
            />

            <Accordion
              type="single"
              value={form.watch("hostingMethode")}
              collapsible
            >
              <AccordionItem value="direct">
                <AccordionContent>
                  <StyledFormField
                    control={form.control}
                    name="port"
                    label="Port"
                    render={({ field }) => (
                      <Input
                        type="number"
                        className="no-spinner w-3/5"
                        placeholder="25071"
                        autoComplete="off"
                        {...field}
                      />
                    )}
                  />
                </AccordionContent>
              </AccordionItem>
            </Accordion>
          </CardContent>
          <CardFooter className="flex w-full justify-center">
            <Button type="submit" className="w-full max-w-3xl">
              Start Server
            </Button>
          </CardFooter>
        </Card>
      </form>
    </Form>
  );
};

export default Host;
