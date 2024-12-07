
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/


export const commands = {
async getAircraftConfigs() : Promise<{ [key in string]: AircraftConfig[] }> {
    return await TAURI_INVOKE("get_aircraft_configs");
},
async saveSettings(username: string, aircraft: string, instructorMode: boolean, streamerMode: boolean) : Promise<null> {
    return await TAURI_INVOKE("save_settings", { username, aircraft, instructorMode, streamerMode });
},
async startServer(method: ConnectionMethod, isIpv6: boolean, port: number | null) : Promise<null> {
    return await TAURI_INVOKE("start_server", { method, isIpv6, port });
},
async disconnect() : Promise<void> {
    await TAURI_INVOKE("disconnect");
},
async transferControl(target: string) : Promise<void> {
    await TAURI_INVOKE("transfer_control", { target });
},
async setObserver(target: string, isObserver: boolean) : Promise<void> {
    await TAURI_INVOKE("set_observer", { target, isObserver });
},
async goObserver() : Promise<void> {
    await TAURI_INVOKE("go_observer");
},
async forceTakeControl() : Promise<void> {
    await TAURI_INVOKE("force_take_control");
},
async getPublicIp() : Promise<string> {
    return await TAURI_INVOKE("get_public_ip");
}
}

/** user-defined events **/


export const events = __makeEvents__<{
clientFailEvent: ClientFailEvent,
gainControlEvent: GainControlEvent,
loseControlEvent: LoseControlEvent,
metricsEvent: MetricsEvent,
serverAttemptEvent: ServerAttemptEvent,
serverFailEvent: ServerFailEvent,
serverStartedEvent: ServerStartedEvent,
setInControlEvent: SetInControlEvent,
setObservingEvent: SetObservingEvent
}>({
clientFailEvent: "client-fail-event",
gainControlEvent: "gain-control-event",
loseControlEvent: "lose-control-event",
metricsEvent: "metrics-event",
serverAttemptEvent: "server-attempt-event",
serverFailEvent: "server-fail-event",
serverStartedEvent: "server-started-event",
setInControlEvent: "set-in-control-event",
setObservingEvent: "set-observing-event"
})

/** user-defined constants **/



/** user-defined types **/

export type AircraftConfig = { name: string; path: string }
export type ClientFailEvent = string
export type ConnectionMethod = "direct" | "relay" | "cloudServer"
export type GainControlEvent = null
export type LoseControlEvent = null
export type MetricsEvent = { sentPackets: number; receivedPackets: number; sentBandwidth: number; receivedBandwidth: number; packetLoss: number; ping: number }
export type ServerAttemptEvent = null
export type ServerFailEvent = string
export type ServerStartedEvent = string
export type SetInControlEvent = string
export type SetObservingEvent = [string, boolean]

/** tauri-specta globals **/

import {
	invoke as TAURI_INVOKE,
	Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
	listen: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
	once: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
	emit: null extends T
		? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
		: (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
	| { status: "ok"; data: T }
	| { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
	mappings: Record<keyof T, string>,
) {
	return new Proxy(
		{} as unknown as {
			[K in keyof T]: __EventObj__<T[K]> & {
				(handle: __WebviewWindow__): __EventObj__<T[K]>;
			};
		},
		{
			get: (_, event) => {
				const name = mappings[event as keyof T];

				return new Proxy((() => {}) as any, {
					apply: (_, __, [window]: [__WebviewWindow__]) => ({
						listen: (arg: any) => window.listen(name, arg),
						once: (arg: any) => window.once(name, arg),
						emit: (arg: any) => window.emit(name, arg),
					}),
					get: (_, command: keyof __EventObj__<any>) => {
						switch (command) {
							case "listen":
								return (arg: any) => TAURI_API_EVENT.listen(name, arg);
							case "once":
								return (arg: any) => TAURI_API_EVENT.once(name, arg);
							case "emit":
								return (arg: any) => TAURI_API_EVENT.emit(name, arg);
						}
					},
				});
			},
		},
	);
}
