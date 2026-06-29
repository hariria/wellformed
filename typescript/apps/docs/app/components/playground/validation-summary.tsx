"use client";

import { AlertCircle, AlertTriangle, CheckCircle2 } from "lucide-react";
import { useMemo } from "react";
import type { FormError, Schema } from "wellformed-ts/ir";
import { validate } from "wellformed-ts/runtime";
import { Badge } from "@/components/ui/badge";

interface ValidationSummaryProps {
  schema: Schema | null;
  values: Record<string, unknown>;
}

export function ValidationSummary({ schema, values }: ValidationSummaryProps) {
  const result = useMemo(() => {
    if (!schema) return null;
    try {
      return validate(schema, values);
    } catch {
      return null;
    }
  }, [schema, values]);

  if (!result) {
    return (
      <div className="text-sm text-muted-foreground">
        No schema loaded for validation.
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        {result.valid ? (
          <>
            <CheckCircle2 className="size-4 text-emerald-600 dark:text-emerald-400" />
            <Badge
              variant="outline"
              className="border-emerald-300 text-emerald-700 dark:border-emerald-700 dark:text-emerald-400"
            >
              Valid
            </Badge>
          </>
        ) : (
          <>
            <AlertCircle className="size-4 text-destructive" />
            <Badge variant="destructive">
              {result.errors.length} error
              {result.errors.length !== 1 ? "s" : ""}
            </Badge>
          </>
        )}
        {result.warnings.length > 0 && (
          <Badge
            variant="outline"
            className="border-amber-300 text-amber-700 dark:border-amber-700 dark:text-amber-400"
          >
            {result.warnings.length} warning
            {result.warnings.length !== 1 ? "s" : ""}
          </Badge>
        )}
      </div>

      {result.errors.length > 0 && (
        <div className="space-y-1.5">
          <h4 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
            Errors
          </h4>
          <ul className="space-y-1">
            {result.errors.map((err) => (
              <ErrorItem
                key={validationErrorKey("error", err)}
                error={err}
                variant="error"
              />
            ))}
          </ul>
        </div>
      )}

      {result.warnings.length > 0 && (
        <div className="space-y-1.5">
          <h4 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
            Warnings
          </h4>
          <ul className="space-y-1">
            {result.warnings.map((warn) => (
              <ErrorItem
                key={validationErrorKey("warning", warn)}
                error={warn}
                variant="warning"
              />
            ))}
          </ul>
        </div>
      )}

      {result.value != null &&
        typeof result.value === "object" &&
        Object.keys(result.value as Record<string, unknown>).length > 0 && (
          <div className="space-y-1.5">
            <h4 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Transformed Values
            </h4>
            <pre className="text-xs font-mono bg-muted/50 rounded-md p-3 overflow-x-auto whitespace-pre-wrap break-words">
              {JSON.stringify(result.value, null, 2)}
            </pre>
          </div>
        )}
    </div>
  );
}

function validationErrorKey(prefix: string, error: FormError): string {
  return `${prefix}:${error.path}:${error.code}:${error.message}`;
}

function ErrorItem({
  error,
  variant,
}: {
  error: FormError;
  variant: "error" | "warning";
}) {
  return (
    <li className="flex items-start gap-2 text-sm">
      {variant === "error" ? (
        <AlertCircle className="size-3.5 mt-0.5 shrink-0 text-destructive" />
      ) : (
        <AlertTriangle className="size-3.5 mt-0.5 shrink-0 text-amber-600 dark:text-amber-400" />
      )}
      <div className="min-w-0">
        <span className="font-mono text-xs text-muted-foreground">
          {error.path || "/"}
        </span>
        <span className="mx-1.5 text-muted-foreground/50">&middot;</span>
        <span className="font-mono text-xs text-muted-foreground">
          {error.code}
        </span>
        <p className="text-sm">{error.message}</p>
      </div>
    </li>
  );
}
