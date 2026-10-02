"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Appear } from "../motion/appear";
import { ErrorNotice } from "./error-notice";

export type QueryLike<Data> = {
  data: Data | undefined;
  error: Error | null;
  refetch: () => unknown;
};

export type LoadedProps<Data> = {
  query: QueryLike<Data>;
  skeleton: ReactNode;
  errorTitle?: string;
  children: (data: Data) => ReactNode;
};

type Datas<Queries extends readonly QueryLike<unknown>[]> = { [Index in keyof Queries]: Queries[Index] extends QueryLike<infer Data> ? Data : never };

export type LoadedAllProps<Queries extends readonly QueryLike<unknown>[]> = {
  queries: Queries;
  skeleton: ReactNode;
  errorTitle?: string;
  children: (...datas: Datas<Queries>) => ReactNode;
};

function Failure({ error, title, onRetry }: { error: Error; title?: string; onRetry: () => void }) {
  const t = useTranslations("errors");
  return <ErrorNotice title={title ?? t("loadFailed")} description={error.message} onRetry={onRetry} />;
}

export function Loaded<Data>({ query, skeleton, errorTitle, children }: LoadedProps<Data>) {
  if (query.data !== undefined) {
    return <Appear>{children(query.data)}</Appear>;
  }
  if (query.error) {
    return <Failure error={query.error} title={errorTitle} onRetry={() => void query.refetch()} />;
  }
  return <>{skeleton}</>;
}

function LoadedAll<const Queries extends readonly QueryLike<unknown>[]>({ queries, skeleton, errorTitle, children }: LoadedAllProps<Queries>) {
  if (queries.every((query) => query.data !== undefined)) {
    return <Appear>{children(...(queries.map((query) => query.data) as Datas<Queries>))}</Appear>;
  }
  const failed = queries.find((query) => query.data === undefined && query.error);
  if (failed?.error) {
    return (
      <Failure
        error={failed.error}
        title={errorTitle}
        onRetry={() => {
          for (const query of queries) {
            void query.refetch();
          }
        }}
      />
    );
  }
  return <>{skeleton}</>;
}

Loaded.all = LoadedAll;
