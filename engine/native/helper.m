#import <Foundation/Foundation.h>
#import <AppKit/AppKit.h>
#import <Security/Security.h>
#import <sys/stat.h>
#import <unistd.h>
#import <libproc.h>
#import <pwd.h>
#import <mach/mach_time.h>

#define PAIR @"/Library/Application Support/CoquiPCCleaner/Helper/pairing.json"
#define SERVICE @"com.coqui.CoquiPCCleaner.Helper"
extern char *coqui_helper_dispatch(const char *, unsigned int, int, int);
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
    if (![p isKindOfClass:NSDictionary.class]) return nil;
    BOOL gate=[p[@"version"] isEqual:@1] && [p[@"mode"] isEqual:@"installationGate"];
    BOOL production=[p[@"version"] isEqual:@2] && [p[@"mode"] isEqual:@"production"];
    if (!gate && !production) return nil;
    for (NSString *key in @[@"clientRequirement", @"helperRequirement"]) {
        NSString *requirement=p[key];
        if (![requirement isKindOfClass:NSString.class] || !coqui_valid_signing_requirement(requirement.UTF8String)) return nil;
    }
    return p;
}
static BOOL installedPeer(NSXPCConnection *connection, NSDictionary *p) {
    if (![p[@"mode"] isEqual:@"production"]) return YES;
    char buffer[PROC_PIDPATHINFO_MAXSIZE];
    if (proc_pidpath(connection.processIdentifier,buffer,sizeof(buffer))<=0) return NO;
    NSString *actual=[[@(buffer) stringByStandardizingPath] stringByResolvingSymlinksInPath];
    NSString *expected=p[@"clientExecutable"];
    NSString *codePath=p[@"applicationPath"];
    NSString *requirement=p[@"applicationRequirement"];
    if ([actual isEqual:[@"/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper" stringByResolvingSymlinksInPath]]) {
        codePath=@"/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper";
        expected=codePath; requirement=p[@"helperRequirement"];
    }
    if (![expected isKindOfClass:NSString.class] || ![actual isEqual:[expected stringByResolvingSymlinksInPath]] || ![codePath isKindOfClass:NSString.class] || ![requirement isKindOfClass:NSString.class]) return NO;
    struct stat metadata;
    if (lstat(codePath.fileSystemRepresentation,&metadata) || metadata.st_uid!=0 || (metadata.st_mode & 0022) || S_ISLNK(metadata.st_mode)) return NO;
    SecStaticCodeRef code=NULL; SecRequirementRef req=NULL;
    OSStatus status=SecStaticCodeCreateWithPath((__bridge CFURLRef)[NSURL fileURLWithPath:codePath],kSecCSDefaultFlags,&code);
    if (status==errSecSuccess) status=SecRequirementCreateWithString((__bridge CFStringRef)requirement,kSecCSDefaultFlags,&req);
    if (status==errSecSuccess) status=SecStaticCodeCheckValidity(code,kSecCSStrictValidate|kSecCSCheckAllArchitectures,req);
    if (req) CFRelease(req); if (code) CFRelease(code);
    return status==errSecSuccess;
}
uint64_t coqui_process_start(int pid) {
    struct proc_bsdinfo info;
    if (proc_pidinfo(pid,PROC_PIDTBSDINFO,0,&info,sizeof(info))!=sizeof(info)) return 0;
    return info.pbi_start_tvsec*1000000ULL+info.pbi_start_tvusec;
}
int coqui_helper_protocol(void) { @autoreleasepool {NSDictionary *p=pairing();return [p[@"mode"] isEqual:@"production"] ? 2 : (p ? 1 : 0);} }

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
    if (![requirement isKindOfClass:NSString.class] || requirement.length > 4096 || !connection.effectiveUserIdentifier || !installedPeer(connection,p)) return NO;
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
    char *response=coqui_helper_dispatch(json.UTF8String,connection.effectiveUserIdentifier,connection.processIdentifier,[pairing()[@"mode"] isEqual:@"production"]);
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
        int count=proc_listallpids(NULL,0);
        if (count<=0) return NULL;
        pid_t *pids=calloc((size_t)count+256,sizeof(pid_t));
        if (!pids) return NULL;
        int found=proc_listallpids(pids,(count+256)*(int)sizeof(pid_t));
        if (found<0 || found>=count+256) {free(pids);return NULL;}
        for(int i=0;i<found;i++) {
            char buffer[PROC_PIDPATHINFO_MAXSIZE];
            if(proc_pidpath(pids[i],buffer,sizeof(buffer))>0) {
                NSString *file=@(buffer); NSRange part=[file rangeOfString:@".app/Contents/"];
                if(part.location!=NSNotFound) [paths addObject:[file substringToIndex:part.location+4]];
            }
        }
        free(pids);
        NSData *data=[NSJSONSerialization dataWithJSONObject:paths options:0 error:NULL];
        return data ? strdup([[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding].UTF8String) : NULL;
    }
}

int coqui_authorization_valid(const char *encoded) {
    @autoreleasepool {
        NSString *text=encoded ? [NSString stringWithUTF8String:encoded] : nil;
        NSData *data=text ? [[NSData alloc] initWithBase64EncodedString:text options:0] : nil;
        if (data.length!=sizeof(AuthorizationExternalForm)) return 0;
        AuthorizationExternalForm form; [data getBytes:&form length:sizeof(form)];
        AuthorizationRef auth=NULL;
        OSStatus status=AuthorizationCreateFromExternalForm(&form,&auth);
        AuthorizationItem item={"system.privilege.admin",0,NULL,0};
        AuthorizationRights rights={1,&item};
        if (status==errAuthorizationSuccess) status=AuthorizationCopyRights(auth,&rights,NULL,kAuthorizationFlagDefaults,NULL);
        if (auth) AuthorizationFree(auth,kAuthorizationFlagDefaults);
        return status==errAuthorizationSuccess;
    }
}
extern int coqui_is_cancelled(const void *);
static NSRecursiveLock *authorizationLock(void) {
    static NSRecursiveLock *lock; static dispatch_once_t once;
    dispatch_once(&once,^{lock=[NSRecursiveLock new];}); return lock;
}
static NSTimeInterval authorizationNow(void) {
    static mach_timebase_info_data_t base; static dispatch_once_t once;
    dispatch_once(&once,^{mach_timebase_info(&base);});
    return (NSTimeInterval)((long double)mach_continuous_time()*base.numer/base.denom/1e9L);
}
static AuthorizationRef sessionAuthorization=NULL;
static NSTimeInterval sessionDeadline=0;
static NSString *sessionPair=nil;
static unsigned int interactiveAuthorizationRequests=0;
static void clearAuthorizationLocked(void) {
    if(sessionAuthorization) AuthorizationFree(sessionAuthorization,kAuthorizationFlagDestroyRights);
    sessionAuthorization=NULL; sessionDeadline=0; sessionPair=nil;
}
static BOOL ensureAuthorizationLocked(const void *cancel) {
    NSDictionary *p=pairing();
    NSString *key=p ? [NSString stringWithFormat:@"%@|%@",p[@"clientRequirement"],p[@"helperRequirement"]] : nil;
    NSTimeInterval now=authorizationNow();
    AuthorizationItem item={"system.privilege.admin",0,NULL,0}; AuthorizationRights rights={1,&item};
    if (!key || ![p[@"mode"] isEqual:@"production"]) {clearAuthorizationLocked();return NO;}
    if (sessionAuthorization && (now>=sessionDeadline || ![sessionPair isEqual:key] || AuthorizationCopyRights(sessionAuthorization,&rights,NULL,kAuthorizationFlagDefaults,NULL)!=errAuthorizationSuccess)) clearAuthorizationLocked();
    if (coqui_is_cancelled(cancel)) return NO;
    if (!sessionAuthorization) {
        OSStatus status=AuthorizationCreate(NULL,kAuthorizationEmptyEnvironment,kAuthorizationFlagDefaults,&sessionAuthorization);
        if(status==errAuthorizationSuccess) {
            interactiveAuthorizationRequests++;
            status=AuthorizationCopyRights(sessionAuthorization,&rights,kAuthorizationEmptyEnvironment,kAuthorizationFlagInteractionAllowed|kAuthorizationFlagExtendRights,NULL);
        }
        if(status!=errAuthorizationSuccess || coqui_is_cancelled(cancel)) {clearAuthorizationLocked();return NO;}
        sessionDeadline=authorizationNow()+15*60;
        sessionPair=[key copy];
    }
    return YES;
}
unsigned int coqui_authorization_begin(const void *cancel) {
    @autoreleasepool {NSRecursiveLock *lock=authorizationLock();[lock lock];@try {
        return ensureAuthorizationLocked(cancel) ? (unsigned int)(sessionDeadline-authorizationNow()) : 0;
    } @catch(NSException *exception){(void)exception;clearAuthorizationLocked();return 0;} @finally {[lock unlock];}}
}
void coqui_authorization_clear(void) {
    @autoreleasepool {NSRecursiveLock *lock=authorizationLock();[lock lock];@try {clearAuthorizationLocked();} @finally {[lock unlock];}}
}
unsigned int coqui_authorization_remaining(void) {
    @autoreleasepool {NSRecursiveLock *lock=authorizationLock();[lock lock];@try {
        NSTimeInterval remaining=sessionDeadline-authorizationNow();
        if(remaining<=0) {clearAuthorizationLocked();return 0;}
        return sessionAuthorization ? (unsigned int)remaining : 0;
    } @finally {[lock unlock];}}
}
unsigned int coqui_authorization_interactive_requests(void) {NSRecursiveLock *lock=authorizationLock();[lock lock];unsigned int result=interactiveAuthorizationRequests;[lock unlock];return result;}
char *coqui_helper_authorized_request(const char *payload, const void *cancel) {
    @autoreleasepool {NSRecursiveLock *lock=authorizationLock();[lock lock];@try {
        if(!ensureAuthorizationLocked(cancel)) return strdup("{\"error\":\"Administrator authorization was cancelled, declined or unavailable; no operation started\"}");
        AuthorizationExternalForm form;
        if (AuthorizationMakeExternalForm(sessionAuthorization,&form)!=errAuthorizationSuccess) {clearAuthorizationLocked();return strdup("{\"error\":\"Administrator session expired; authorize again\"}");}
        NSData *raw=[[NSString stringWithUTF8String:payload] dataUsingEncoding:NSUTF8StringEncoding];
        NSMutableDictionary *request=[[NSJSONSerialization JSONObjectWithData:raw options:0 error:NULL] mutableCopy];
        if (![request isKindOfClass:NSMutableDictionary.class]) return strdup("{\"error\":\"Invalid typed authorization request\"}");
        request[@"authorization"]=[[NSData dataWithBytes:&form length:sizeof(form)] base64EncodedStringWithOptions:0];
        NSData *encoded=[NSJSONSerialization dataWithJSONObject:request options:0 error:NULL];
        if(coqui_is_cancelled(cancel)) return strdup("{\"error\":\"Operation cancelled before start\"}");
        return coqui_helper_request([[NSString alloc]initWithData:encoded encoding:NSUTF8StringEncoding].UTF8String);
    } @catch(NSException *exception) {(void)exception;clearAuthorizationLocked();return strdup("{\"error\":\"Administrator authorization failed; no automatic retry\"}");} @finally {[lock unlock];} }
}
char *coqui_user_trash_directory(const char *path) {
    @autoreleasepool { @try {
        NSURL *url=[NSFileManager.defaultManager URLForDirectory:NSTrashDirectory inDomain:NSUserDomainMask appropriateForURL:[NSURL fileURLWithPath:[NSString stringWithUTF8String:path]] create:YES error:NULL];
        return url.path ? strdup(url.path.UTF8String) : NULL;
    } @catch(NSException *exception){(void)exception;return NULL;} }
}
