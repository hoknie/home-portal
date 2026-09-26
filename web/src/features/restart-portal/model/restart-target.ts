import type { Network } from "@/entities/network";

export type Place = { origin: string; protocol: string; hostname: string; port: string };

const DEFAULT_PORTS: Record<string, number> = { "http:": 80, "https:": 443 };

function portOf(place: Place) {
  return place.port === "" ? (DEFAULT_PORTS[place.protocol] ?? 0) : Number(place.port);
}

function effectivePort(address: string) {
  return Number(address.slice(address.lastIndexOf(":") + 1));
}

export function restartTarget(place: Place, network?: Network | null) {
  if (!network || network.effective.overridden || !network.restart_required) {
    return place.origin;
  }
  const current = portOf(place);
  const direct = current === effectivePort(network.effective.address);
  if (!direct || network.configured.port === current) {
    return place.origin;
  }
  return `${place.protocol}//${place.hostname}:${network.configured.port}`;
}
