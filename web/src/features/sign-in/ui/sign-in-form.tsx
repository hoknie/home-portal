"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useQueryClient } from "@tanstack/react-query";
import { LogIn } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { useForm } from "react-hook-form";

import { type Credentials, credentialsSchema, sessionKey, signIn } from "@/entities/session";
import { ThrottledError, UnauthorizedError } from "@/shared/api";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input } from "@/shared/ui/primitives";

export type SignInFormProps = { onSignedIn: () => void };

export function SignInForm({ onSignedIn }: SignInFormProps) {
  const t = useTranslations("login");
  const client = useQueryClient();
  const [problem, setProblem] = useState<string | null>(null);
  const form = useForm<Credentials>({ resolver: zodResolver(credentialsSchema), defaultValues: { name: "", password: "" } });

  const submit = form.handleSubmit(async (credentials) => {
    setProblem(null);
    try {
      const session = await signIn(credentials);
      client.setQueryData(sessionKey, session);
      onSignedIn();
    } catch (error) {
      if (error instanceof ThrottledError) {
        setProblem(t("throttled", { seconds: error.retryAfterSeconds }));
      } else if (error instanceof UnauthorizedError) {
        setProblem(t("invalid"));
        form.resetField("password");
      } else {
        setProblem(t("failed"));
      }
    }
  });

  const pending = form.formState.isSubmitting;
  return (
    <form onSubmit={submit} className="grid gap-5" noValidate>
      {problem ? <ErrorNotice title={problem} /> : null}
      <FormField id="name" label={t("name")} error={form.formState.errors.name?.message}>
        <Input id="name" autoComplete="username" autoFocus aria-invalid={Boolean(form.formState.errors.name)} {...form.register("name")} />
      </FormField>
      <FormField id="password" label={t("password")} error={form.formState.errors.password?.message}>
        <Input
          id="password"
          type="password"
          autoComplete="current-password"
          aria-invalid={Boolean(form.formState.errors.password)}
          {...form.register("password")}
        />
      </FormField>
      <Button type="submit" size="lg" disabled={pending}>
        <LogIn aria-hidden />
        {pending ? t("submitting") : t("submit")}
      </Button>
    </form>
  );
}
