import type { Schema } from "wellformed-ts/ir";

export interface Preset {
  name: string;
  description: string;
  schema: Schema;
}

const contactForm: Preset = {
  name: "Contact Form",
  description: "String validation, transforms, optional fields, and enums",
  schema: {
    version: "1.0",
    id: "contact-form",
    title: "Contact Form",
    description: "Basic contact information form",
    root: {
      type: "object",
      properties: {
        firstName: {
          type: "string",
          required: true,
          transforms: [{ fn: "trim" }],
          constraints: [
            {
              pred: { type: "min_len", len: 1 },
              error: {
                code: "REQUIRED",
                message: "First name is required",
              },
            },
          ],
        },
        lastName: {
          type: "string",
          required: true,
          transforms: [{ fn: "trim" }],
          constraints: [
            {
              pred: { type: "min_len", len: 1 },
              error: {
                code: "REQUIRED",
                message: "Last name is required",
              },
            },
          ],
        },
        email: {
          type: "string",
          required: true,
          transforms: [{ fn: "trim" }, { fn: "lower" }],
          constraints: [
            {
              pred: { type: "call", name: "is_email" },
              error: {
                code: "INVALID_EMAIL",
                message: "Must be a valid email address",
              },
            },
          ],
        },
        phone: {
          type: "string",
          required: false,
          transforms: [{ fn: "phone_us" }],
          constraints: [
            {
              pred: { type: "call", name: "phone_number_us" },
              error: {
                code: "INVALID_PHONE",
                message: "Must be a valid US phone number",
              },
            },
          ],
        },
        preferredContact: {
          type: "enum",
          required: true,
          values: ["email", "phone", "mail"],
        },
      },
    },
  },
};

const taxInformation: Preset = {
  name: "Tax Information",
  description: "Domain predicates, nested objects, and transforms",
  schema: {
    version: "1.0",
    id: "tax-info",
    title: "Tax Information",
    description: "W-9 style tax information form",
    root: {
      type: "object",
      properties: {
        tinType: {
          type: "enum",
          required: true,
          values: ["ssn", "ein"],
        },
        tin: {
          type: "string",
          required: true,
          transforms: [{ fn: "digits_only" }],
          constraints: [
            {
              pred: { type: "call", name: "is_ssn" },
              error: {
                code: "INVALID_TIN",
                message: "Must be a valid SSN (9 digits)",
              },
            },
          ],
        },
        name: {
          type: "string",
          required: true,
          transforms: [{ fn: "trim" }],
          constraints: [
            {
              pred: { type: "min_len", len: 1 },
              error: {
                code: "REQUIRED",
                message: "Name is required",
              },
            },
          ],
        },
        address: {
          type: "object",
          required: true,
          properties: {
            street: {
              type: "string",
              required: true,
              transforms: [{ fn: "trim" }],
              constraints: [
                {
                  pred: { type: "min_len", len: 1 },
                  error: {
                    code: "REQUIRED",
                    message: "Street address is required",
                  },
                },
              ],
            },
            city: {
              type: "string",
              required: true,
              transforms: [{ fn: "trim" }],
              constraints: [
                {
                  pred: { type: "min_len", len: 1 },
                  error: {
                    code: "REQUIRED",
                    message: "City is required",
                  },
                },
              ],
            },
            state: {
              type: "string",
              required: true,
              transforms: [{ fn: "trim" }, { fn: "upper" }],
              constraints: [
                {
                  pred: { type: "call", name: "is_us_state" },
                  error: {
                    code: "INVALID_STATE",
                    message: "Must be a valid US state code (e.g. CA, NY)",
                  },
                },
              ],
            },
            zip: {
              type: "string",
              required: true,
              transforms: [{ fn: "digits_only" }],
              constraints: [
                {
                  pred: { type: "call", name: "is_us_zip" },
                  error: {
                    code: "INVALID_ZIP",
                    message: "Must be a valid US ZIP code (5 or 9 digits)",
                  },
                },
              ],
            },
          },
        },
        taxYear: {
          type: "integer",
          required: true,
          constraints: [
            {
              pred: { type: "range", min: 2020, max: 2026 },
              error: {
                code: "INVALID_TAX_YEAR",
                message: "Tax year must be between 2020 and 2026",
              },
            },
          ],
        },
      },
    },
  },
};

const financial: Preset = {
  name: "Financial",
  description: "Money type, arrays, and financial validators",
  schema: {
    version: "1.0",
    id: "financial",
    title: "Financial Account",
    description: "Bank account with transaction history",
    root: {
      type: "object",
      properties: {
        accountHolder: {
          type: "string",
          required: true,
          transforms: [{ fn: "trim" }],
          constraints: [
            {
              pred: { type: "min_len", len: 1 },
              error: {
                code: "REQUIRED",
                message: "Account holder name is required",
              },
            },
          ],
        },
        accountType: {
          type: "enum",
          required: true,
          values: ["checking", "savings", "money_market"],
        },
        routingNumber: {
          type: "string",
          required: true,
          transforms: [{ fn: "digits_only" }],
          constraints: [
            {
              pred: { type: "call", name: "is_aba_routing" },
              error: {
                code: "INVALID_ROUTING",
                message: "Must be a valid ABA routing number",
              },
            },
          ],
        },
        balance: {
          type: "money",
          required: true,
          constraints: [
            {
              pred: { type: "call", name: "is_non_negative" },
              error: {
                code: "NEGATIVE_BALANCE",
                message: "Balance cannot be negative",
              },
            },
          ],
        },
        transactions: {
          type: "array",
          required: false,
          items: {
            type: "object",
            properties: {
              date: {
                type: "string",
                required: true,
                constraints: [
                  {
                    pred: { type: "call", name: "is_date" },
                    error: {
                      code: "INVALID_DATE",
                      message: "Must be a valid date (MM/DD/YYYY)",
                    },
                  },
                ],
              },
              amount: {
                type: "money",
                required: true,
              },
              description: {
                type: "string",
                required: true,
                transforms: [{ fn: "trim" }],
                constraints: [
                  {
                    pred: { type: "min_len", len: 1 },
                    error: {
                      code: "REQUIRED",
                      message: "Description is required",
                    },
                  },
                ],
              },
            },
          },
        },
      },
    },
  },
};

const crossFieldRules: Preset = {
  name: "Cross-field Rules",
  description: "eq_fields, gte_field, sum_equals cross-field validation",
  schema: {
    version: "1.0",
    id: "cross-field",
    title: "Cross-field Validation",
    description: "Demonstrates cross-field validation rules",
    root: {
      type: "object",
      properties: {
        password: {
          type: "string",
          required: true,
          constraints: [
            {
              pred: { type: "min_len", len: 8 },
              error: {
                code: "PASSWORD_TOO_SHORT",
                message: "Password must be at least 8 characters",
              },
            },
          ],
        },
        confirmPassword: {
          type: "string",
          required: true,
        },
        startDate: {
          type: "string",
          required: true,
          constraints: [
            {
              pred: { type: "call", name: "is_date" },
              error: {
                code: "INVALID_DATE",
                message: "Must be a valid date",
              },
            },
          ],
        },
        endDate: {
          type: "string",
          required: true,
          constraints: [
            {
              pred: { type: "call", name: "is_date" },
              error: {
                code: "INVALID_DATE",
                message: "Must be a valid date",
              },
            },
          ],
        },
        subtotal: {
          type: "number",
          required: true,
        },
        tax: {
          type: "number",
          required: true,
        },
        shipping: {
          type: "number",
          required: true,
        },
        total: {
          type: "number",
          required: true,
        },
      },
      rules: [
        {
          pred: {
            type: "eq_fields",
            left: "/password",
            right: "/confirmPassword",
          },
          error: {
            code: "PASSWORDS_MISMATCH",
            message: "Passwords must match",
            severity: "error",
          },
        },
        {
          pred: {
            type: "gte_field",
            left: "/endDate",
            right: "/startDate",
          },
          error: {
            code: "DATE_RANGE_INVALID",
            message: "End date must be on or after start date",
            severity: "error",
          },
        },
        {
          pred: {
            type: "sum_equals",
            paths: ["/subtotal", "/tax", "/shipping"],
            target: "/total",
          },
          error: {
            code: "SUM_MISMATCH",
            message: "Subtotal, tax, and shipping must add up to the total",
            severity: "error",
          },
        },
      ],
    },
  },
};

export const presets: Preset[] = [
  contactForm,
  taxInformation,
  financial,
  crossFieldRules,
];
