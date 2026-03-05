import type { DesktopSize } from '../interfaces/DesktopSize';
import { Config } from './Config';
import type { Extension } from '../interfaces/Extension';

/**
 * Builder class for creating Config objects with a fluent interface.
 *
 * @example
 * ```typescript
 * const configBuilder = new ConfigBuilder(createExtensionFunction);
 * const config = configBuilder
 *   .withDestination(destination)
 *   .withProxyAddress(proxyAddress)
 *   .withAuthToken(authToken)
 *   ...
 *   .build();
 * ```
 */
export class ConfigBuilder {
    private username: string = '';
    private password: string = '';
    private destination: string = '';
    private proxyAddress: string = '';
    private serverDomain: string = '';
    private authToken: string = '';
    private desktopSize?: DesktopSize;
    private extensions: Extension[] = [];
    private _bytesReceivedCallback?: (bytes: number) => void;
    private _bytesSentCallback?: (bytes: number) => void;

    /**
     * Creates a new ConfigBuilder instance.
     */
    constructor() {}

    /**
     * Optional parameter
     *
     * @param username - The username to use for authentication
     * @returns The builder instance for method chaining
     */
    withUsername(username: string): ConfigBuilder {
        this.username = username;
        return this;
    }

    /**
     * Optional parameter
     *
     * @param password - The password for authentication
     * @returns The builder instance for method chaining
     */
    withPassword(password: string): ConfigBuilder {
        this.password = password;
        return this;
    }

    /**
     * Required parameter
     *
     * @param destination - The destination address to connect to
     * @returns The builder instance for method chaining
     */
    withDestination(destination: string): ConfigBuilder {
        this.destination = destination;
        return this;
    }

    /**
     * Required parameter
     *
     * @param proxyAddress - The address of the proxy server
     * @returns The builder instance for method chaining
     */
    withProxyAddress(proxyAddress: string): ConfigBuilder {
        this.proxyAddress = proxyAddress;
        return this;
    }

    /**
     * Optional parameter
     *
     * @param serverDomain - The server domain to connect to
     * @returns The builder instance for method chaining
     */
    withServerDomain(serverDomain: string): ConfigBuilder {
        this.serverDomain = serverDomain;
        return this;
    }

    /**
     * Required parameter
     *
     * @param authToken - JWT token to connect to the proxy
     * @returns The builder instance for method chaining
     */
    withAuthToken(authToken: string): ConfigBuilder {
        this.authToken = authToken;
        return this;
    }

    /**
     * Optional parameter
     *
     * @param ext - The extension
     * @returns The builder instance for method chaining
     */
    withExtension(ext: Extension): ConfigBuilder {
        this.extensions.push(ext);
        return this;
    }

    /**
     * Optional
     *
     * @param desktopSize - The desktop size configuration object
     * @returns The builder instance for method chaining
     */
    withDesktopSize(desktopSize: DesktopSize): ConfigBuilder {
        this.desktopSize = desktopSize;
        return this;
    }

    /**
     * Optional callback for tracking bytes received (RX)
     *
     * @param callback - Function called with number of bytes received
     * @returns The builder instance for method chaining
     */
    bytesReceivedCallback(callback: (bytes: number) => void): ConfigBuilder {
        this._bytesReceivedCallback = callback;
        return this;
    }

    /**
     * Optional callback for tracking bytes sent (TX)
     *
     * @param callback - Function called with number of bytes sent
     * @returns The builder instance for method chaining
     */
    bytesSentCallback(callback: (bytes: number) => void): ConfigBuilder {
        this._bytesSentCallback = callback;
        return this;
    }

    /**
     * Get the bytes received callback
     */
    getBytesReceivedCallback(): ((bytes: number) => void) | undefined {
        return this._bytesReceivedCallback;
    }

    /**
     * Get the bytes sent callback
     */
    getBytesSentCallback(): ((bytes: number) => void) | undefined {
        return this._bytesSentCallback;
    }

    /**
     * Builds a new Config instance.
     *
     * @throws {Error} If required parameters (destination, proxyAddress, authToken) are not set
     * @returns A new Config instance with the configured values
     */
    build(): Config {
        if (this.destination === '') {
            throw new Error('destination has to be specified');
        }
        if (this.proxyAddress === '') {
            throw new Error('proxy address has to be specified');
        }
        if (this.authToken === '') {
            throw new Error('authentication token has to be specified');
        }
        const userData = { username: this.username, password: this.password };
        const proxyData = { address: this.proxyAddress, authToken: this.authToken };

        const configOptions = {
            destination: this.destination,
            serverDomain: this.serverDomain,
            extensions: this.extensions,
            desktopSize: this.desktopSize,
            bytesReceivedCallback: this._bytesReceivedCallback,
            bytesSentCallback: this._bytesSentCallback,
        };

        return new Config(userData, proxyData, configOptions);
    }
}
