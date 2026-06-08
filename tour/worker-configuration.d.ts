declare namespace Cloudflare {
	interface Env {
		ASSETS: Fetcher;
	}
}

interface Env extends Cloudflare.Env {}

interface Fetcher {
	fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

interface ExecutionContext {
	passThroughOnException(): void;
	waitUntil(promise: Promise<unknown>): void;
}

interface IncomingRequestCfProperties {
	[key: string]: unknown;
}
