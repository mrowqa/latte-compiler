; ModuleID = 'lib/runtime.cpp'
source_filename = "lib/runtime.cpp"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct._IO_FILE = type { i32, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, %struct._IO_marker*, %struct._IO_FILE*, i32, i32, i64, i16, i8, [1 x i8], i8*, i64, %struct._IO_codecvt*, %struct._IO_wide_data*, %struct._IO_FILE*, i8*, i64, i32, [20 x i8] }
%struct._IO_marker = type opaque
%struct._IO_codecvt = type opaque
%struct._IO_wide_data = type opaque

@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@.str.1 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@.str.2 = private unnamed_addr constant [15 x i8] c"runtime error\0A\00", align 1
@stdin = external local_unnamed_addr global %struct._IO_FILE*, align 8
@.str.3 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1

; Function Attrs: sspstrong uwtable
define dso_local void @printInt(i32) local_unnamed_addr #0 {
  %2 = tail call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @.str, i64 0, i64 0), i32 %0) #9
  ret void
}

declare i32 @printf(i8*, ...) local_unnamed_addr #1

; Function Attrs: sspstrong uwtable
define dso_local void @printString(i8*) local_unnamed_addr #0 {
  %2 = tail call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @.str.1, i64 0, i64 0), i8* %0) #9
  ret void
}

; Function Attrs: noreturn sspstrong uwtable
define dso_local void @error() local_unnamed_addr #2 {
  %1 = tail call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([15 x i8], [15 x i8]* @.str.2, i64 0, i64 0)) #9
  tail call void @exit(i32 1) #10
  unreachable
}

; Function Attrs: noreturn nounwind
declare void @exit(i32) local_unnamed_addr #3

; Function Attrs: sspstrong uwtable
define dso_local i32 @readInt() local_unnamed_addr #0 {
  %1 = alloca i32, align 4
  %2 = bitcast i32* %1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %2) #11
  %3 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @.str, i64 0, i64 0), i32* nonnull %1) #9
  %4 = load i32, i32* %1, align 4, !tbaa !4
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %2) #11
  ret i32 %4
}

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.start.p0i8(i64, i8* nocapture) #4

declare i32 @scanf(i8*, ...) local_unnamed_addr #1

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.end.p0i8(i64, i8* nocapture) #4

; Function Attrs: sspstrong uwtable
define dso_local i8* @readString() local_unnamed_addr #0 {
  %1 = alloca i8*, align 8
  %2 = alloca i64, align 8
  %3 = bitcast i8** %1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %3) #11
  store i8* null, i8** %1, align 8, !tbaa !8
  %4 = bitcast i64* %2 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %4) #11
  store i64 0, i64* %2, align 8, !tbaa !10
  %5 = load %struct._IO_FILE*, %struct._IO_FILE** @stdin, align 8, !tbaa !8
  %6 = call i64 @__getdelim(i8** nonnull %1, i64* nonnull %2, i32 10, %struct._IO_FILE* %5) #9
  %7 = icmp eq i64 %6, 0
  br i1 %7, label %16, label %8

; <label>:8:                                      ; preds = %0
  %9 = load i8*, i8** %1, align 8, !tbaa !8
  %10 = add i64 %6, -1
  %11 = getelementptr inbounds i8, i8* %9, i64 %10
  %12 = load i8, i8* %11, align 1, !tbaa !12
  %13 = icmp eq i8 %12, 10
  br i1 %13, label %14, label %16

; <label>:14:                                     ; preds = %8
  store i8 0, i8* %11, align 1, !tbaa !12
  %15 = load i8*, i8** %1, align 8, !tbaa !8
  br label %16

; <label>:16:                                     ; preds = %8, %14, %0
  %17 = phi i8* [ getelementptr inbounds ([1 x i8], [1 x i8]* @.str.3, i64 0, i64 0), %0 ], [ %15, %14 ], [ %9, %8 ]
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %4) #11
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %3) #11
  ret i8* %17
}

; Function Attrs: nounwind sspstrong uwtable
define dso_local i8* @_bltn_string_concat(i8*, i8*) local_unnamed_addr #5 {
  %3 = tail call i64 @strlen(i8* %0) #12
  %4 = tail call i64 @strlen(i8* %1) #12
  %5 = add i64 %3, 1
  %6 = add i64 %5, %4
  %7 = tail call noalias i8* @malloc(i64 %6) #13
  %8 = tail call i8* @strcpy(i8* %7, i8* %0) #13
  %9 = tail call i8* @strcat(i8* %7, i8* %1) #13
  ret i8* %7
}

; Function Attrs: nounwind readonly
declare i64 @strlen(i8*) local_unnamed_addr #6

; Function Attrs: nounwind
declare noalias i8* @malloc(i64) local_unnamed_addr #7

; Function Attrs: nounwind
declare i8* @strcpy(i8*, i8*) local_unnamed_addr #7

; Function Attrs: nounwind
declare i8* @strcat(i8*, i8*) local_unnamed_addr #7

; Function Attrs: nounwind readonly sspstrong uwtable
define dso_local zeroext i1 @_bltn_string_eq(i8* readonly, i8* readonly) local_unnamed_addr #8 {
  %3 = tail call i32 @strcmp(i8* %0, i8* %1) #12
  %4 = icmp eq i32 %3, 0
  ret i1 %4
}

; Function Attrs: nounwind readonly
declare i32 @strcmp(i8*, i8*) local_unnamed_addr #6

; Function Attrs: nounwind readonly sspstrong uwtable
define dso_local zeroext i1 @_bltn_string_ne(i8* readonly, i8* readonly) local_unnamed_addr #8 {
  %3 = tail call i32 @strcmp(i8* %0, i8* %1) #12
  %4 = icmp ne i32 %3, 0
  ret i1 %4
}

; Function Attrs: sspstrong uwtable
define dso_local i8* @_bltn_malloc(i32) local_unnamed_addr #0 {
  %2 = icmp slt i32 %0, 0
  br i1 %2, label %3, label %4

; <label>:3:                                      ; preds = %1
  tail call void @error() #9
  unreachable

; <label>:4:                                      ; preds = %1
  %5 = sext i32 %0 to i64
  %6 = tail call noalias i8* @malloc(i64 %5) #13
  %7 = icmp eq i8* %6, null
  br i1 %7, label %8, label %9

; <label>:8:                                      ; preds = %4
  tail call void @error() #9
  unreachable

; <label>:9:                                      ; preds = %4
  %10 = tail call i8* @memset(i8* nonnull %6, i32 0, i64 %5) #13
  ret i8* %6
}

; Function Attrs: nounwind
declare i8* @memset(i8*, i32, i64) local_unnamed_addr #7

declare i64 @__getdelim(i8**, i64*, i32, %struct._IO_FILE*) local_unnamed_addr #1

attributes #0 = { sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { noreturn sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { noreturn nounwind "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { argmemonly nounwind }
attributes #5 = { nounwind sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #6 = { nounwind readonly "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #7 = { nounwind "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #8 = { nounwind readonly sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #9 = { nobuiltin }
attributes #10 = { nobuiltin noreturn nounwind }
attributes #11 = { nounwind }
attributes #12 = { nobuiltin nounwind readonly }
attributes #13 = { nobuiltin nounwind }

!llvm.module.flags = !{!0, !1, !2}
!llvm.ident = !{!3}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{!"clang version 7.0.1 (tags/RELEASE_701/final)"}
!4 = !{!5, !5, i64 0}
!5 = !{!"int", !6, i64 0}
!6 = !{!"omnipotent char", !7, i64 0}
!7 = !{!"Simple C++ TBAA"}
!8 = !{!9, !9, i64 0}
!9 = !{!"any pointer", !6, i64 0}
!10 = !{!11, !11, i64 0}
!11 = !{!"long", !6, i64 0}
!12 = !{!6, !6, i64 0}
