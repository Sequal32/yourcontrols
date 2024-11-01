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

interface SimplifiedFormFieldProps
  extends React.ComponentProps<typeof FormField> {
  // TODO: remove any
  control: Control<any>;
  label: string;
  description?: string;
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
            <FormDescription>{description}</FormDescription>
          </FormMessage>
        </div>
        <FormControl>{formControl(r)}</FormControl>
      </FormItem>
    )}
  />
);

export default StyledFormField;
