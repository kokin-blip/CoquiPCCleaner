#import <Foundation/Foundation.h>
#import <AppKit/AppKit.h>
#import <Security/Security.h>
#import <sys/stat.h>
#import <unistd.h>

#define PAIR @"/Library/Application Support/CoquiPCCleaner/Helper/pairing.json"
#define SERVICE @"com.coqui.CoquiPCCleaner.Helper"
extern char *coqui_helper_dispatch(const char *, unsigned int);
extern void coqui_helper_free(char *);

int coqui_valid_signing_requirement(const char *text) {
    @autoreleasepool {
        if (!text) return 0;
        NSString *value=[NSString stringWithUTF8String:text];
        if (!value || value.length > 4096) return 0;
        SecRequirementRef requirement=NULL;
        OSStatus status=SecRequirementCreateWithString((__bridge CFStringRef)value,kSecCSDefaultFlags,&requirement);
        if (requirement) CFRelease(requirement);
        return status == errSecSuccess;
    }
}
static NSDictionary *pairing(void) {
    struct stat s;
    NSString *parent = [PAIR stringByDeletingLastPathComponent];
    if (lstat(parent.fileSystemRepresentation,&s) || !S_ISDIR(s.st_mode) || s.st_uid != 0 || (s.st_mode & 0022)) return nil;
    if (![[parent stringByResolvingSymlinksInPath] isEqual:parent]) return nil;
    if (lstat(PAIR.fileSystemRepresentation,&s) || !S_ISREG(s.st_mode) || s.st_uid != 0 || (s.st_mode & 0022) || s.st_size > 16384) return nil;
    NSData *data=[NSData dataWithContentsOfFile:PAIR];
    id p=data ? [NSJSONSerialization JSONObjectWithData:data options:0 error:NULL] : nil;
    if (![p isKindOfClass:NSDictionary.class] || ![p[@"version"] isEqual:@1] || ![p[@"mode"] isEqual:@"installationGate"]) return nil;
    for (NSString *key in @[@"clientRequirement", @"helperRequirement"]) {
        NSString *requirement=p[key];
        if (![requirement isKindOfClass:NSString.class] || !coqui_valid_signing_requirement(requirement.UTF8String)) return nil;
    }
    return p;
}
@protocol CoquiHelperProtocol
- (void)request:(NSData *)request reply:(void (^)(NSData *))reply;
@end
@interface CoquiHelper : NSObject <CoquiHelperProtocol, NSXPCListenerDelegate>
@end
@implementation CoquiHelper
- (BOOL)listener:(NSXPCListener *)listener shouldAcceptNewConnection:(NSXPCConnection *)connection {
    @try {
    NSDictionary *p=pairing();
    NSString *requirement=p[@"clientRequirement"];
    if (![requirement isKindOfClass:NSString.class] || requirement.length > 4096 || !connection.effectiveUserIdentifier) return NO;
    [connection setCodeSigningRequirement:requirement];
    connection.exportedInterface=[NSXPCInterface interfaceWithProtocol:@protocol(CoquiHelperProtocol)];
    connection.exportedObject=self;
    [connection activate];
    return YES;
    } @catch (NSException *exception) { (void)exception; return NO; }
}
- (void)request:(NSData *)request reply:(void (^)(NSData *))reply {
    NSXPCConnection *connection=NSXPCConnection.currentConnection;
    if (!connection || request.length > 1024*1024) { reply([@"{\"error\":\"Request denied\"}" dataUsingEncoding:NSUTF8StringEncoding]); return; }
    NSString *json=[[NSString alloc] initWithData:request encoding:NSUTF8StringEncoding];
    if (!json || memchr(request.bytes,0,request.length) != NULL) { reply([@"{\"error\":\"Invalid request\"}" dataUsingEncoding:NSUTF8StringEncoding]); return; }
    char *response=coqui_helper_dispatch(json.UTF8String,connection.effectiveUserIdentifier);
    NSData *data=response ? [[NSString stringWithUTF8String:response] dataUsingEncoding:NSUTF8StringEncoding] : nil;
    coqui_helper_free(response);
    reply(data ?: [@"{\"error\":\"Helper failed\"}" dataUsingEncoding:NSUTF8StringEncoding]);
}
@end
int coqui_helper_listen(void) {
    @autoreleasepool {
        @try {
        if (geteuid()!=0 || !pairing()) return 77;
        CoquiHelper *delegate=[CoquiHelper new];
        NSXPCListener *listener=[[NSXPCListener alloc] initWithMachServiceName:SERVICE];
        listener.delegate=delegate;
        [listener activate];
        [[NSRunLoop currentRunLoop] run];
        } @catch (NSException *exception) { (void)exception; return 78; }
    }
    return 0;
}
char *coqui_helper_request(const char *payload) {
    @autoreleasepool {
        @try {
        NSDictionary *p=pairing();
        NSString *requirement=p[@"helperRequirement"];
        if (![requirement isKindOfClass:NSString.class]) return strdup("{\"error\":\"Administrator helper pairing is absent or invalid; install the matching package\"}");
        NSXPCConnection *connection=[[NSXPCConnection alloc] initWithMachServiceName:SERVICE options:NSXPCConnectionPrivileged];
        [connection setCodeSigningRequirement:requirement];
        connection.remoteObjectInterface=[NSXPCInterface interfaceWithProtocol:@protocol(CoquiHelperProtocol)];
        dispatch_semaphore_t signal=dispatch_semaphore_create(0);
        __block NSData *result=nil;
        __block BOOL completed=NO;
        NSLock *lock=[NSLock new];
        void (^finish)(NSData *)=^(NSData *data){
            [lock lock];
            if (!completed) {completed=YES;result=data;dispatch_semaphore_signal(signal);}
            [lock unlock];
        };
        [connection activate];
        id<CoquiHelperProtocol> proxy=[connection remoteObjectProxyWithErrorHandler:^(NSError *error) {
            (void)error;
            finish([@"{\"error\":\"Helper connection rejected or unavailable; install the matching package\"}" dataUsingEncoding:NSUTF8StringEncoding]);
        }];
        [proxy request:[[NSString stringWithUTF8String:payload] dataUsingEncoding:NSUTF8StringEncoding] reply:^(NSData *data) {finish(data);}];
        long waited=dispatch_semaphore_wait(signal,dispatch_time(DISPATCH_TIME_NOW,30*NSEC_PER_SEC));
        [lock lock];
        NSData *response=result;
        completed=YES;
        [lock unlock];
        [connection invalidate];
        if (waited || !response) return strdup("{\"error\":\"Helper timed out; no automatic retry\"}");
        NSString *json=[[NSString alloc] initWithData:response encoding:NSUTF8StringEncoding];
        return strdup(json.UTF8String ?: "{\"error\":\"Invalid helper response\"}");
        } @catch (NSException *exception) {
            (void)exception;
            return strdup("{\"error\":\"Helper configuration or connection rejected; install the matching package\"}");
        }
    }
}

char *coqui_running_applications(void) {
    @autoreleasepool {
        NSMutableArray *paths=[NSMutableArray new];
        for (NSRunningApplication *app in NSWorkspace.sharedWorkspace.runningApplications) {
            if (app.bundleURL.path) [paths addObject:app.bundleURL.path];
        }
        NSData *data=[NSJSONSerialization dataWithJSONObject:paths options:0 error:NULL];
        return data ? strdup([[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding].UTF8String) : NULL;
    }
}
