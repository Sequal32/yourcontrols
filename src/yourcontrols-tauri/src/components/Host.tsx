import React from "react";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@ui/card";
import { Button } from "@ui/button";
import { ToggleGroup, ToggleGroupItem } from "@ui/toggle-group";
import { Form } from "@ui/form";
import { useForm } from "react-hook-form";
import StyledFormField from "@/components/StyledFormField";
import { commands } from "@/types/bindings";

// TODO: FormSchema
const Host: React.FC = () => {
  const form = useForm({
    defaultValues: {
      ipVersion: "ipv4",
      hostingMode: "p2p",
    },
  });

  // TODO
  const onSubmit = (data: any) => {
    console.log(data);
    commands.startServer("relay");
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
              name="hostingMode"
              label="Hosting Mode"
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
                  <ToggleGroupItem value="p2p">Cloud P2P</ToggleGroupItem>
                  <ToggleGroupItem value="host">Cloud Host</ToggleGroupItem>
                  <ToggleGroupItem value="direct">Direct</ToggleGroupItem>
                </ToggleGroup>
              )}
            />
          </CardContent>
          <CardFooter>
            <Button type="submit" className="w-full">
              Start Server
            </Button>
          </CardFooter>
        </Card>
      </form>
    </Form>
  );
};

export default Host;
