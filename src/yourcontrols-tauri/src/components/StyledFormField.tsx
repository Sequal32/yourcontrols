import React from "react";
import { Control } from "react-hook-form";
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@ui/form";
import { Skeleton } from "@ui/skeleton";

interface SimplifiedFormFieldProps
  extends React.ComponentProps<typeof FormField> {
  // TODO: remove any
  control: Control<any>;
  label: string;
  description?: string | null;
  loadingDescription?: boolean;
}

const StyledFormField: React.FC<SimplifiedFormFieldProps> = ({
  label,
  description,
  render: formControl,
  ...props
}) => (
  <FormField
    {...props}
    render={(r) => (
      <FormItem className="flex h-20 items-center justify-between space-y-0 rounded-lg border p-4">
        <div className="space-y-0.5">
          <FormLabel className="text-base">{label}</FormLabel>
          <FormMessage>
            {description !== null ? (
              <FormDescription>{description}</FormDescription>
            ) : (
              <Skeleton className="h-[20px] w-[90px]" />
            )}
          </FormMessage>
        </div>
        <FormControl>{formControl(r)}</FormControl>
      </FormItem>
    )}
  />
);

export default StyledFormField;
