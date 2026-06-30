"use client";

import { Plus, Trash2 } from "lucide-react";
import { Controller, useFieldArray, useFormContext } from "react-hook-form";
import type { PropertySchema, TypeSchema } from "wellformed-ts/ir";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface FieldRendererProps {
  name: string;
  label: string;
  schema: TypeSchema;
  required?: boolean;
}

export function FieldRenderer({
  name,
  label,
  schema,
  required,
}: FieldRendererProps) {
  switch (schema.type) {
    case "string":
      return (
        <StringField
          name={name}
          label={label}
          required={required}
          description={schema.description}
        />
      );
    case "number":
    case "integer":
    case "int32":
    case "int64":
    case "uint32":
    case "uint64":
    case "money":
    case "decimal":
    case "currency":
    case "percentage":
      return (
        <NumberField
          name={name}
          label={label}
          required={required}
          isInteger={
            schema.type === "integer" ||
            schema.type === "int32" ||
            schema.type === "int64" ||
            schema.type === "uint32" ||
            schema.type === "uint64"
          }
          description={schema.description}
        />
      );
    case "boolean":
      return (
        <BooleanField
          name={name}
          label={label}
          description={schema.description}
        />
      );
    case "enum":
      return (
        <EnumField
          name={name}
          label={label}
          values={schema.values}
          required={required}
        />
      );
    case "date":
      return (
        <DateField
          name={name}
          label={label}
          required={required}
          description={schema.description}
        />
      );
    case "object":
      return (
        <ObjectField
          name={name}
          label={label}
          properties={schema.properties ?? {}}
        />
      );
    case "array":
      return <ArrayField name={name} label={label} itemSchema={schema.items} />;
    default:
      return <StringField name={name} label={label} required={required} />;
  }
}

function StringField({
  name,
  label,
  required,
  description,
}: {
  name: string;
  label: string;
  required?: boolean;
  description?: string;
}) {
  const {
    register,
    formState: { errors },
  } = useFormContext();
  const error = getNestedError(errors, name);

  return (
    <Field>
      <FieldLabel required={required}>{label}</FieldLabel>
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
      <Input {...register(name)} placeholder={label} />
      <FieldError message={error?.message as string} />
    </Field>
  );
}

function NumberField({
  name,
  label,
  required,
  isInteger,
  description,
}: {
  name: string;
  label: string;
  required?: boolean;
  isInteger?: boolean;
  description?: string;
}) {
  const {
    register,
    formState: { errors },
  } = useFormContext();
  const error = getNestedError(errors, name);

  return (
    <Field>
      <FieldLabel required={required}>{label}</FieldLabel>
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
      <Input
        type="number"
        step={isInteger ? "1" : "any"}
        {...register(name, { valueAsNumber: true })}
        placeholder={label}
      />
      <FieldError message={error?.message as string} />
    </Field>
  );
}

function BooleanField({
  name,
  label,
  description,
}: {
  name: string;
  label: string;
  description?: string;
}) {
  const { control } = useFormContext();

  return (
    <Field>
      <div className="flex items-center gap-2">
        <Controller
          control={control}
          name={name}
          render={({ field }) => (
            <Checkbox
              checked={field.value ?? false}
              onCheckedChange={field.onChange}
            />
          )}
        />
        <FieldLabel>{label}</FieldLabel>
      </div>
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
    </Field>
  );
}

function EnumField({
  name,
  label,
  values,
  required,
}: {
  name: string;
  label: string;
  values: unknown[];
  required?: boolean;
}) {
  const {
    control,
    formState: { errors },
  } = useFormContext();
  const error = getNestedError(errors, name);

  return (
    <Field>
      <FieldLabel required={required}>{label}</FieldLabel>
      <Controller
        control={control}
        name={name}
        render={({ field }) => (
          <Select value={field.value ?? ""} onValueChange={field.onChange}>
            <SelectTrigger>
              <SelectValue placeholder={`Select ${label.toLowerCase()}`} />
            </SelectTrigger>
            <SelectContent>
              {values.map((v) => (
                <SelectItem key={String(v)} value={String(v)}>
                  {String(v)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      />
      <FieldError message={error?.message as string} />
    </Field>
  );
}

function DateField({
  name,
  label,
  required,
  description,
}: {
  name: string;
  label: string;
  required?: boolean;
  description?: string;
}) {
  const {
    register,
    formState: { errors },
  } = useFormContext();
  const error = getNestedError(errors, name);

  return (
    <Field>
      <FieldLabel required={required}>{label}</FieldLabel>
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
      <Input type="date" {...register(name)} />
      <FieldError message={error?.message as string} />
    </Field>
  );
}

function ObjectField({
  name,
  label,
  properties,
}: {
  name: string;
  label: string;
  properties: Record<string, PropertySchema>;
}) {
  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-sm">{label}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {Object.entries(properties).map(([key, propSchema]) => {
          const isRequired = propSchema.required !== false;
          return (
            <FieldRenderer
              key={key}
              name={`${name}.${key}`}
              label={formatLabel(key)}
              schema={propSchema as TypeSchema}
              required={isRequired}
            />
          );
        })}
      </CardContent>
    </Card>
  );
}

function ArrayField({
  name,
  label,
  itemSchema,
}: {
  name: string;
  label: string;
  itemSchema: TypeSchema;
}) {
  const { control } = useFormContext();
  const { fields, append, remove } = useFieldArray({
    control,
    name,
  });

  const defaultItem =
    itemSchema.type === "object"
      ? Object.fromEntries(
          Object.keys(itemSchema.properties ?? {}).map((k) => [k, ""]),
        )
      : "";

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">{label}</CardTitle>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => append(defaultItem)}
          >
            <Plus className="size-3.5" />
            Add
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {fields.length === 0 && (
          <p className="text-sm text-muted-foreground">
            No items. Click &ldquo;Add&rdquo; to add one.
          </p>
        )}
        {fields.map((field, index) => (
          <div key={field.id} className="relative border rounded-lg p-3 pt-8">
            <div className="absolute top-2 right-2 flex items-center gap-2">
              <span className="text-xs text-muted-foreground">
                #{index + 1}
              </span>
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                onClick={() => remove(index)}
              >
                <Trash2 className="size-3.5" />
              </Button>
            </div>
            {itemSchema.type === "object" ? (
              <div className="space-y-3">
                {Object.entries(itemSchema.properties ?? {}).map(
                  ([key, propSchema]) => {
                    const isRequired =
                      (propSchema as PropertySchema).required !== false;
                    return (
                      <FieldRenderer
                        key={key}
                        name={`${name}.${index}.${key}`}
                        label={formatLabel(key)}
                        schema={propSchema as TypeSchema}
                        required={isRequired}
                      />
                    );
                  },
                )}
              </div>
            ) : (
              <FieldRenderer
                name={`${name}.${index}`}
                label={`Item ${index + 1}`}
                schema={itemSchema}
              />
            )}
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

function formatLabel(key: string): string {
  return key
    .replace(/([A-Z])/g, " $1")
    .replace(/^./, (s) => s.toUpperCase())
    .trim();
}

// biome-ignore lint/suspicious/noExplicitAny: RHF errors are dynamic
function getNestedError(errors: any, path: string): any {
  const parts = path.split(".");
  let current = errors;
  for (const part of parts) {
    if (!current) return undefined;
    current = current[part];
  }
  return current;
}
