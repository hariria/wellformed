"use client";

import { useCallback, useEffect, useMemo, useRef } from "react";
import { FormProvider, useForm } from "react-hook-form";
import type { PropertySchema, Schema, TypeSchema } from "wellformed-ts/ir";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { wellformedResolver } from "@/lib/wellformed-resolver";
import { FieldRenderer } from "./field-renderer";
import { ValidationSummary } from "./validation-summary";

interface FormPreviewProps {
  schema: Schema | null;
  parseError: string | null;
}

export function FormPreview({ schema, parseError }: FormPreviewProps) {
  const prevSchemaId = useRef<string | undefined>(undefined);

  const resolver = useMemo(() => {
    if (!schema) return undefined;
    return wellformedResolver(schema);
  }, [schema]);

  const methods = useForm({
    resolver,
    mode: "onChange",
    defaultValues: {},
  });

  const { reset, watch, trigger } = methods;
  const values = watch();

  // Reset form when schema changes
  useEffect(() => {
    const currentId = schema?.id ?? schema?.title;
    if (currentId !== prevSchemaId.current) {
      prevSchemaId.current = currentId;
      reset({});
    }
  }, [schema, reset]);

  // Re-trigger validation when resolver changes (schema edited)
  const triggerValidation = useCallback(() => {
    trigger();
  }, [trigger]);

  useEffect(() => {
    if (resolver) {
      triggerValidation();
    }
  }, [resolver, triggerValidation]);

  if (parseError) {
    return (
      <Card className="border-destructive/50">
        <CardHeader>
          <CardTitle className="text-destructive">Schema Error</CardTitle>
          <CardDescription>
            Fix the JSON to see the form preview
          </CardDescription>
        </CardHeader>
        <CardContent>
          <pre className="text-sm text-destructive whitespace-pre-wrap break-words font-mono">
            {parseError}
          </pre>
        </CardContent>
      </Card>
    );
  }

  if (!schema) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>No Schema</CardTitle>
          <CardDescription>
            Select a preset or enter a valid schema JSON to see the form
            preview.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  const rootSchema = schema.root;
  const properties =
    rootSchema.type === "object" ? (rootSchema.properties ?? {}) : {};

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>{schema.title ?? "Form Preview"}</CardTitle>
          {schema.description && (
            <CardDescription>{schema.description}</CardDescription>
          )}
        </CardHeader>
        <CardContent>
          <FormProvider {...methods}>
            <form
              onSubmit={methods.handleSubmit(() => {})}
              className="space-y-4"
            >
              {Object.entries(properties).map(([key, propSchema]) => {
                const isRequired =
                  (propSchema as PropertySchema).required !== false;
                return (
                  <FieldRenderer
                    key={key}
                    name={key}
                    label={formatLabel(key)}
                    schema={propSchema as TypeSchema}
                    required={isRequired}
                  />
                );
              })}
            </form>
          </FormProvider>
        </CardContent>
      </Card>

      <Separator />

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Validation Result</CardTitle>
        </CardHeader>
        <CardContent>
          <ValidationSummary schema={schema} values={values} />
        </CardContent>
      </Card>
    </div>
  );
}

function formatLabel(key: string): string {
  return key
    .replace(/([A-Z])/g, " $1")
    .replace(/^./, (s) => s.toUpperCase())
    .trim();
}
