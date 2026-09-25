import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Proxy, proxySchema } from "@/entities/proxy";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";
import { Toaster } from "@/shared/ui/primitives";

import { CaddyControl } from "./caddy-control";

function sample(change: (proxy: Proxy) => void = () => undefined) {
  const proxy = proxySchema.parse(structuredClone(apiSamples.proxy));
  change(proxy);
  return proxy;
}

function answer(change: (proxy: Proxy) => void) {
  return jsonResponse(sample(change));
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("downloads caddy when none is installed", async () => {
  const fetch = vi.fn(async () =>
    answer((proxy) => {
      proxy.caddy.installed = null;
      proxy.caddy.managed = false;
      proxy.caddy.download.state = "downloading";
    }),
  );
  vi.stubGlobal("fetch", fetch);
  const initial = sample((proxy) => {
    proxy.caddy.installed = null;
    proxy.caddy.managed = false;
  });
  renderWithProviders(
    <>
      <CaddyControl proxy={initial} revision='"r1"' />
      <Toaster />
    </>,
  );
  expect(screen.getByText("не скачан")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Запустить" })).toBeDisabled();
  await userEvent.click(screen.getByRole("button", { name: "Скачать последнюю версию" }));
  expect(fetch).toHaveBeenCalledWith("/api/proxy/caddy/download", expect.objectContaining({ method: "POST" }));
  expect(await screen.findByText("Загрузка Caddy началась")).toBeInTheDocument();
});

it("starts an installed caddy with the revision it loaded", async () => {
  const fetch = vi.fn(async () =>
    answer((proxy) => {
      proxy.caddy.managed = true;
    }),
  );
  vi.stubGlobal("fetch", fetch);
  const installed = sample((proxy) => {
    proxy.caddy.managed = false;
  });
  renderWithProviders(<CaddyControl proxy={installed} revision='"r2"' />);
  expect(screen.getByText("2.11.4")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Запустить" }));
  const started = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(started[0]).toBe("/api/proxy/caddy/start");
  expect((started[1].headers as Record<string, string>)["If-Match"]).toBe('"r2"');
});

it("shows the progress of a download", () => {
  renderWithProviders(
    <CaddyControl
      proxy={sample((proxy) => {
        proxy.caddy.download.state = "downloading";
      })}
      revision={null}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent("Скачиваем");
  expect(screen.getByRole("button", { name: "Скачать последнюю версию" })).toBeDisabled();
});

it("shows the reason a download failed", () => {
  renderWithProviders(
    <CaddyControl
      proxy={sample((proxy) => {
        proxy.caddy.download = { state: "failed", error: "the SHA-512 of caddy.tar.gz does not match" };
      })}
      revision={null}
    />,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("SHA-512");
});

it("shows the log of a managed caddy that does not answer", () => {
  renderWithProviders(
    <CaddyControl
      proxy={sample((proxy) => {
        proxy.reachable = false;
        proxy.caddy.log = ["listen tcp :443: bind: permission denied"];
      })}
      revision={null}
    />,
  );
  expect(screen.getByLabelText("Журнал Caddy")).toHaveTextContent("permission denied");
  expect(screen.getByRole("button", { name: "Остановить" })).toBeEnabled();
});

it("shows where caddy comes from and that the archive is verified", () => {
  renderWithProviders(<CaddyControl proxy={sample()} revision='"r1"' />);
  expect(screen.getByText("https://api.github.com/repos/caddyserver/caddy/releases/latest")).toBeInTheDocument();
  expect(screen.getByText("caddy_*_mac_arm64.tar.gz")).toBeInTheDocument();
  expect(screen.getByText(/SHA-512/)).toBeInTheDocument();
  expect(screen.getByText(/caddy_2\.11\.4_mac_arm64\.tar\.gz$/)).toBeInTheDocument();
});

it("names a pinned version on the download button and in the archive", () => {
  const pinned = sample((proxy) => {
    proxy.caddy.version = "2.10.2";
    proxy.caddy.release_url = "https://api.github.com/repos/caddyserver/caddy/releases/tags/v2.10.2";
  });
  renderWithProviders(<CaddyControl proxy={pinned} revision='"r1"' />);
  expect(screen.getByRole("button", { name: "Скачать Caddy 2.10.2" })).toBeEnabled();
  expect(screen.getByText("caddy_2.10.2_mac_arm64.tar.gz")).toBeInTheDocument();
});

it("disables the download on a machine caddy cannot be downloaded for", () => {
  const foreign = sample((proxy) => {
    proxy.caddy.platform = null;
    proxy.caddy.platform_error = "Caddy cannot be downloaded for the operating system windows; install it by hand";
  });
  renderWithProviders(<CaddyControl proxy={foreign} revision='"r1"' />);
  expect(screen.getByRole("button", { name: "Скачать последнюю версию" })).toBeDisabled();
  expect(screen.getByText(/operating system windows/)).toBeInTheDocument();
});
