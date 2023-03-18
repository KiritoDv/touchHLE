/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSBundle`.

use super::{ns_string, NSUInteger};
use crate::bundle::Bundle;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, ClassExports, HostObject,
};

#[derive(Default)]
pub struct State {
    main_bundle: Option<id>,
}

struct NSBundleHostObject {
    /// If this is [None], this is the main bundle's NSBundle instance and the
    /// [Bundle] is stored in [crate::Environment], not here.
    _bundle: Option<Bundle>,
    /// NSString with bundle path.
    bundle_path: id,
    /// NSURL with bundle path. [None] if not created yet.
    bundle_url: Option<id>,
}
impl HostObject for NSBundleHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSBundle: NSObject

+ (id)mainBundle {
    if let Some(bundle) = env.framework_state.foundation.ns_bundle.main_bundle {
        bundle
    } else {
        let bundle_path = env.bundle.bundle_path().as_str().to_string();
        let bundle_path = ns_string::from_rust_string(env, bundle_path);
        let host_object = NSBundleHostObject {
            _bundle: None,
            bundle_path,
            bundle_url: None,
        };
        let new = env.objc.alloc_object(
            this,
            Box::new(host_object),
            &mut env.mem
        );
        env.framework_state.foundation.ns_bundle.main_bundle = Some(new);
        new
   }
}

- (())dealloc {
    let &NSBundleHostObject { bundle_url, .. } = env.objc.borrow(this);
    if let Some(bundle_url) = bundle_url {
        release(env, bundle_url);
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)bundlePath {
    env.objc.borrow::<NSBundleHostObject>(this).bundle_path
}
- (id)bundleURL {
    if let Some(url) = env.objc.borrow::<NSBundleHostObject>(this).bundle_url {
        url
    } else {
        let bundle_path: id = msg![env; this bundlePath];
        let new: id = msg_class![env; NSURL alloc];
        let new: id = msg![env; new initFileURLWithPath:bundle_path];
        env.objc.borrow_mut::<NSBundleHostObject>(this).bundle_url = Some(new);
        new
    }
}

- (id)resourcePath {
    // This seems to be the same as the bundle path. The iPhone OS bundle
    // structure is a lot flatter than the macOS one.
    msg![env; this bundlePath]
}
- (id)resourceURL {
    // This seems to be the same as the bundle path. The iPhone OS bundle
    // structure is a lot flatter than the macOS one.
    msg![env; this bundleURL]
}

- (id)pathForResource:(id)name // NSString*
               ofType:(id)extension // NSString*
          inDirectory:(id)directory { // NSString*
    assert!(directory == nil || {
        let length: NSUInteger = msg![env; directory length];
        length == 0
    }); // TODO
    assert!(name != nil); // TODO

    let name: id = if extension != nil {
        msg![env; name stringByAppendingPathExtension:extension]
    } else {
        name
    };

    let base_path: id = msg![env; this resourcePath];
    // FIXME: localized resource handling?
    // FIXME: return nil if path does not exist
    msg![env; base_path stringByAppendingPathComponent:name]
}
- (id)pathForResource:(id)name // NSString*
               ofType:(id)extension { // NSString*
    msg![env; this pathForResource:name ofType:extension inDirectory:nil]
}
- (id)URLForResource:(id)name // NSString*
       withExtension:(id)extension { // NSString *
   let path_string: id = msg![env; this pathForResource:name ofType:extension];
   let path_url: id = msg_class![env; NSURL alloc];
   let path_url: id = msg![env; path_url initFileURLWithPath:path_string];
   autorelease(env, path_url)
}
- (id)infoDictionary {
    // log(env.bundle.info_plist().);
    nil
}
- (id) localizedStringForKey:(id)_key // NSString*
                       value:(id)_value // NSString*
                       table:(id)_table { // NSString*
    ns_string::get_static_str(env, "unknown")
}

// TODO: constructors, more accessors

@end

};
