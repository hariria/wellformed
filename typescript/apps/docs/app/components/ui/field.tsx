"use client";

import type * as React from "react";
import { cn } from "@/lib/utils";
import { Label } from "./label";

interface FieldProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

function Field({ className, ...props }: FieldProps) {
  return (
    <div data-slot="field" className={cn("space-y-2", className)} {...props} />
  );
}

interface FieldLabelProps extends React.ComponentProps<typeof Label> {
  required?: boolean;
}

function FieldLabel({
  required,
  children,
  className,
  ...props
}: FieldLabelProps) {
  return (
    <Label className={className} {...props}>
      {children}
      {required && <span className="text-destructive">*</span>}
    </Label>
  );
}

interface FieldDescriptionProps
  extends React.HTMLAttributes<HTMLParagraphElement> {
  children: React.ReactNode;
}

function FieldDescription({ className, ...props }: FieldDescriptionProps) {
  return (
    <p
      data-slot="field-description"
      className={cn("text-xs text-muted-foreground", className)}
      {...props}
    />
  );
}

interface FieldErrorProps extends React.HTMLAttributes<HTMLParagraphElement> {
  message?: string;
}

function FieldError({ message, className, ...props }: FieldErrorProps) {
  if (!message) return null;

  return (
    <p
      data-slot="field-error"
      className={cn("text-xs text-destructive", className)}
      role="alert"
      {...props}
    >
      {message}
    </p>
  );
}

export { Field, FieldDescription, FieldError, FieldLabel };
