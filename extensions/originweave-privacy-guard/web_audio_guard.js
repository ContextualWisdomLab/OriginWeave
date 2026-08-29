"use strict";

// Injected by Manifest V3 at document_start in the MAIN world so page scripts
// cannot capture the native constructors before the privacy policy applies.
(() => {
  const allowedOrigins = [
    /* ORIGINWEAVE_ALLOWED_WEB_AUDIO_ORIGINS */
  ];
  const currentOrigin = globalThis.location.origin;
  for (let index = 0; index < allowedOrigins.length; index += 1) {
    if (allowedOrigins[index] === currentOrigin) {
      return;
    }
  }

  const blockedConstructorNames = Object.freeze([
    "AudioContext",
    "webkitAudioContext",
    "OfflineAudioContext",
    "webkitOfflineAudioContext",
    "AudioWorkletNode"
  ]);
  const blockedMessage = "Web Audio disabled by OriginWeave privacy policy";

  const createBlockedConstructor = () => {
    const blockedConstructor = function originweaveBlockedWebAudioConstructor() {
      throw new DOMException(blockedMessage, "NotAllowedError");
    };
    Object.freeze(blockedConstructor.prototype);
    return Object.freeze(blockedConstructor);
  };

  for (const constructorName of blockedConstructorNames) {
    if (!(constructorName in globalThis)) {
      continue;
    }
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, constructorName);
    Object.defineProperty(globalThis, constructorName, {
      value: createBlockedConstructor(),
      enumerable: descriptor?.enumerable ?? false,
      configurable: false,
      writable: false
    });
  }
})();
