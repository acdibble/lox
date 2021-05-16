import LoxClass from "./LoxClass.ts";
import type LoxInstance from "./LoxInstance.ts";
import NativeFunction from "./NativeFunction.ts";

interface LoxMapInstance extends LoxInstance {
  storage?: Map<any, any>;
}

class LoxMap extends LoxClass {
  constructor() {
    super(null, null, "Map", {
      init: new class extends NativeFunction {
        implementation(this: LoxMapInstance): void {
          this.storage = new Map();
        }
      }({ args: [] }),
      set: new class extends NativeFunction {
        implementation(this: LoxMapInstance, key: any, value: any): void {
          this.storage!.set(key, value);
        }
      }({ args: ["key", "value"] }),
      get: new class extends NativeFunction {
        implementation(this: LoxMapInstance, key: any): void {
          return this.storage!.get(key) ?? null;
        }
      }({ args: ["key"] }),
    });
  }
}

export default LoxMap;
