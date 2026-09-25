export type FieldError = { field: string; message: string };

export class RequestError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "RequestError";
  }
}

export class UnauthorizedError extends RequestError {
  constructor(message: string) {
    super(401, message);
    this.name = "UnauthorizedError";
  }
}

export class ConflictError extends RequestError {
  constructor(message: string) {
    super(409, message);
    this.name = "ConflictError";
  }
}

export class ValidationError extends RequestError {
  constructor(readonly fields: FieldError[]) {
    super(422, fields.map((field) => `${field.field}: ${field.message}`).join("; "));
    this.name = "ValidationError";
  }
}

export class ThrottledError extends RequestError {
  constructor(
    readonly retryAfterSeconds: number,
    message: string,
  ) {
    super(429, message);
    this.name = "ThrottledError";
  }
}

export class UnreachableError extends Error {
  constructor(cause: unknown) {
    super(cause instanceof Error ? cause.message : String(cause));
    this.name = "UnreachableError";
  }
}
