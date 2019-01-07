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
  %1 = alloca i8*, align 8
  %2 = alloca i64, align 8
  %3 = bitcast i8** %1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %3) #11
  store i8* null, i8** %1, align 8, !tbaa !4
  %4 = bitcast i64* %2 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %4) #11
  store i64 0, i64* %2, align 8, !tbaa !8
  %5 = load %struct._IO_FILE*, %struct._IO_FILE** @stdin, align 8, !tbaa !4
  %6 = call i64 @__getdelim(i8** nonnull %1, i64* nonnull %2, i32 10, %struct._IO_FILE* %5) #9
  %7 = icmp eq i64 %6, 0
  br i1 %7, label %8, label %9

; <label>:8:                                      ; preds = %0
  call void @error() #9
  unreachable

; <label>:9:                                      ; preds = %0
  %10 = load i8*, i8** %1, align 8, !tbaa !4
  %11 = icmp sgt i64 %6, 0
  br i1 %11, label %12, label %25

; <label>:12:                                     ; preds = %9, %20
  %13 = phi i8* [ %21, %20 ], [ %10, %9 ]
  %14 = load i8, i8* %13, align 1, !tbaa !10
  %15 = sext i8 %14 to i32
  %16 = call i32 @isspace(i32 %15) #12
  %17 = icmp eq i32 %16, 0
  br i1 %17, label %18, label %20

; <label>:18:                                     ; preds = %12
  %19 = load i8*, i8** %1, align 8, !tbaa !4
  br label %25

; <label>:20:                                     ; preds = %12
  %21 = getelementptr inbounds i8, i8* %13, i64 1
  %22 = load i8*, i8** %1, align 8, !tbaa !4
  %23 = getelementptr inbounds i8, i8* %22, i64 %6
  %24 = icmp ult i8* %21, %23
  br i1 %24, label %12, label %25

; <label>:25:                                     ; preds = %20, %18, %9
  %26 = phi i8* [ %10, %9 ], [ %19, %18 ], [ %22, %20 ]
  %27 = phi i8* [ %10, %9 ], [ %13, %18 ], [ %21, %20 ]
  %28 = getelementptr inbounds i8, i8* %26, i64 %6
  %29 = icmp ult i8* %27, %28
  br i1 %29, label %30, label %39

; <label>:30:                                     ; preds = %25
  %31 = load i8, i8* %27, align 1, !tbaa !10
  %32 = icmp eq i8 %31, 45
  br i1 %32, label %33, label %35

; <label>:33:                                     ; preds = %30
  %34 = getelementptr inbounds i8, i8* %27, i64 1
  br label %39

; <label>:35:                                     ; preds = %30
  %36 = icmp eq i8 %31, 43
  %37 = getelementptr inbounds i8, i8* %27, i64 1
  %38 = select i1 %36, i8* %37, i8* %27
  br label %39

; <label>:39:                                     ; preds = %25, %35, %33
  %40 = phi i8* [ %34, %33 ], [ %38, %35 ], [ %27, %25 ]
  %41 = icmp ult i8* %40, %28
  br i1 %41, label %42, label %56

; <label>:42:                                     ; preds = %39, %51
  %43 = phi i8* [ %52, %51 ], [ %40, %39 ]
  %44 = load i8, i8* %43, align 1, !tbaa !10
  %45 = sext i8 %44 to i32
  %46 = call i32 @isspace(i32 %45) #12
  %47 = icmp eq i32 %46, 0
  br i1 %47, label %48, label %51

; <label>:48:                                     ; preds = %42
  %49 = load i8*, i8** %1, align 8, !tbaa !4
  %50 = getelementptr inbounds i8, i8* %49, i64 %6
  br label %56

; <label>:51:                                     ; preds = %42
  %52 = getelementptr inbounds i8, i8* %43, i64 1
  %53 = load i8*, i8** %1, align 8, !tbaa !4
  %54 = getelementptr inbounds i8, i8* %53, i64 %6
  %55 = icmp ult i8* %52, %54
  br i1 %55, label %42, label %56

; <label>:56:                                     ; preds = %51, %48, %39
  %57 = phi i8* [ %28, %39 ], [ %50, %48 ], [ %54, %51 ]
  %58 = phi i8* [ %40, %39 ], [ %43, %48 ], [ %52, %51 ]
  %59 = icmp ult i8* %58, %57
  br i1 %59, label %60, label %69

; <label>:60:                                     ; preds = %56
  %61 = load i8, i8* %58, align 1, !tbaa !10
  %62 = sext i8 %61 to i32
  %63 = call i32 @isdigit(i32 %62) #12
  %64 = icmp eq i32 %63, 0
  br i1 %64, label %69, label %65

; <label>:65:                                     ; preds = %60
  %66 = load i8*, i8** %1, align 8, !tbaa !4
  %67 = getelementptr inbounds i8, i8* %66, i64 %6
  %68 = icmp ult i8* %58, %67
  br i1 %68, label %70, label %84

; <label>:69:                                     ; preds = %60, %56
  call void @error() #9
  unreachable

; <label>:70:                                     ; preds = %65, %79
  %71 = phi i8* [ %80, %79 ], [ %58, %65 ]
  %72 = load i8, i8* %71, align 1, !tbaa !10
  %73 = sext i8 %72 to i32
  %74 = call i32 @isdigit(i32 %73) #12
  %75 = icmp eq i32 %74, 0
  br i1 %75, label %76, label %79

; <label>:76:                                     ; preds = %70
  %77 = load i8*, i8** %1, align 8, !tbaa !4
  %78 = getelementptr inbounds i8, i8* %77, i64 %6
  br label %84

; <label>:79:                                     ; preds = %70
  %80 = getelementptr inbounds i8, i8* %71, i64 1
  %81 = load i8*, i8** %1, align 8, !tbaa !4
  %82 = getelementptr inbounds i8, i8* %81, i64 %6
  %83 = icmp ult i8* %80, %82
  br i1 %83, label %70, label %84

; <label>:84:                                     ; preds = %79, %76, %65
  %85 = phi i8* [ %67, %65 ], [ %78, %76 ], [ %82, %79 ]
  %86 = phi i8* [ %66, %65 ], [ %77, %76 ], [ %81, %79 ]
  %87 = phi i8* [ %58, %65 ], [ %71, %76 ], [ %80, %79 ]
  %88 = icmp ult i8* %87, %85
  br i1 %88, label %89, label %103

; <label>:89:                                     ; preds = %84, %98
  %90 = phi i8* [ %99, %98 ], [ %87, %84 ]
  %91 = load i8, i8* %90, align 1, !tbaa !10
  %92 = sext i8 %91 to i32
  %93 = call i32 @isspace(i32 %92) #12
  %94 = icmp eq i32 %93, 0
  br i1 %94, label %95, label %98

; <label>:95:                                     ; preds = %89
  %96 = load i8*, i8** %1, align 8, !tbaa !4
  %97 = getelementptr inbounds i8, i8* %96, i64 %6
  br label %103

; <label>:98:                                     ; preds = %89
  %99 = getelementptr inbounds i8, i8* %90, i64 1
  %100 = load i8*, i8** %1, align 8, !tbaa !4
  %101 = getelementptr inbounds i8, i8* %100, i64 %6
  %102 = icmp ult i8* %99, %101
  br i1 %102, label %89, label %103

; <label>:103:                                    ; preds = %98, %95, %84
  %104 = phi i8* [ %85, %84 ], [ %97, %95 ], [ %101, %98 ]
  %105 = phi i8* [ %86, %84 ], [ %96, %95 ], [ %100, %98 ]
  %106 = phi i8* [ %87, %84 ], [ %90, %95 ], [ %99, %98 ]
  %107 = icmp eq i8* %106, %104
  br i1 %107, label %109, label %108

; <label>:108:                                    ; preds = %103
  call void @error() #9
  unreachable

; <label>:109:                                    ; preds = %103
  %110 = call i64 @strtol(i8* nonnull %105, i8** null, i32 10) #12
  %111 = trunc i64 %110 to i32
  call void @free(i8* nonnull %105) #12
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %4) #11
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %3) #11
  ret i32 %111
}

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.start.p0i8(i64, i8* nocapture) #4

; Function Attrs: nounwind
declare i32 @isspace(i32) local_unnamed_addr #5

; Function Attrs: nounwind
declare i32 @isdigit(i32) local_unnamed_addr #5

; Function Attrs: nounwind
declare void @free(i8*) local_unnamed_addr #5

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.end.p0i8(i64, i8* nocapture) #4

; Function Attrs: sspstrong uwtable
define dso_local i8* @readString() local_unnamed_addr #0 {
  %1 = alloca i8*, align 8
  %2 = alloca i64, align 8
  %3 = bitcast i8** %1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %3) #11
  store i8* null, i8** %1, align 8, !tbaa !4
  %4 = bitcast i64* %2 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %4) #11
  store i64 0, i64* %2, align 8, !tbaa !8
  %5 = load %struct._IO_FILE*, %struct._IO_FILE** @stdin, align 8, !tbaa !4
  %6 = call i64 @__getdelim(i8** nonnull %1, i64* nonnull %2, i32 10, %struct._IO_FILE* %5) #9
  %7 = icmp eq i64 %6, 0
  br i1 %7, label %16, label %8

; <label>:8:                                      ; preds = %0
  %9 = load i8*, i8** %1, align 8, !tbaa !4
  %10 = add i64 %6, -1
  %11 = getelementptr inbounds i8, i8* %9, i64 %10
  %12 = load i8, i8* %11, align 1, !tbaa !10
  %13 = icmp eq i8 %12, 10
  br i1 %13, label %14, label %16

; <label>:14:                                     ; preds = %8
  store i8 0, i8* %11, align 1, !tbaa !10
  %15 = load i8*, i8** %1, align 8, !tbaa !4
  br label %16

; <label>:16:                                     ; preds = %8, %14, %0
  %17 = phi i8* [ getelementptr inbounds ([1 x i8], [1 x i8]* @.str.3, i64 0, i64 0), %0 ], [ %15, %14 ], [ %9, %8 ]
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %4) #11
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %3) #11
  ret i8* %17
}

; Function Attrs: nounwind sspstrong uwtable
define dso_local i8* @_bltn_string_concat(i8*, i8*) local_unnamed_addr #6 {
  %3 = tail call i64 @strlen(i8* %0) #13
  %4 = tail call i64 @strlen(i8* %1) #13
  %5 = add i64 %3, 1
  %6 = add i64 %5, %4
  %7 = tail call noalias i8* @malloc(i64 %6) #12
  %8 = tail call i8* @strcpy(i8* %7, i8* %0) #12
  %9 = tail call i8* @strcat(i8* %7, i8* %1) #12
  ret i8* %7
}

; Function Attrs: nounwind readonly
declare i64 @strlen(i8*) local_unnamed_addr #7

; Function Attrs: nounwind
declare noalias i8* @malloc(i64) local_unnamed_addr #5

; Function Attrs: nounwind
declare i8* @strcpy(i8*, i8*) local_unnamed_addr #5

; Function Attrs: nounwind
declare i8* @strcat(i8*, i8*) local_unnamed_addr #5

; Function Attrs: nounwind readonly sspstrong uwtable
define dso_local zeroext i1 @_bltn_string_eq(i8* readonly, i8* readonly) local_unnamed_addr #8 {
  %3 = tail call i32 @strcmp(i8* %0, i8* %1) #13
  %4 = icmp eq i32 %3, 0
  ret i1 %4
}

; Function Attrs: nounwind readonly
declare i32 @strcmp(i8*, i8*) local_unnamed_addr #7

; Function Attrs: nounwind readonly sspstrong uwtable
define dso_local zeroext i1 @_bltn_string_ne(i8* readonly, i8* readonly) local_unnamed_addr #8 {
  %3 = tail call i32 @strcmp(i8* %0, i8* %1) #13
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
  %6 = tail call noalias i8* @malloc(i64 %5) #12
  %7 = icmp eq i8* %6, null
  br i1 %7, label %8, label %9

; <label>:8:                                      ; preds = %4
  tail call void @error() #9
  unreachable

; <label>:9:                                      ; preds = %4
  %10 = tail call i8* @memset(i8* nonnull %6, i32 0, i64 %5) #12
  ret i8* %6
}

; Function Attrs: nounwind
declare i8* @memset(i8*, i32, i64) local_unnamed_addr #5

declare i64 @__getdelim(i8**, i64*, i32, %struct._IO_FILE*) local_unnamed_addr #1

; Function Attrs: nounwind
declare i64 @strtol(i8*, i8**, i32) local_unnamed_addr #5

attributes #0 = { sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { noreturn sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { noreturn nounwind "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { argmemonly nounwind }
attributes #5 = { nounwind "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #6 = { nounwind sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #7 = { nounwind readonly "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #8 = { nounwind readonly sspstrong uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #9 = { nobuiltin }
attributes #10 = { nobuiltin noreturn nounwind }
attributes #11 = { nounwind }
attributes #12 = { nobuiltin nounwind }
attributes #13 = { nobuiltin nounwind readonly }

!llvm.module.flags = !{!0, !1, !2}
!llvm.ident = !{!3}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{!"clang version 7.0.1 (tags/RELEASE_701/final)"}
!4 = !{!5, !5, i64 0}
!5 = !{!"any pointer", !6, i64 0}
!6 = !{!"omnipotent char", !7, i64 0}
!7 = !{!"Simple C++ TBAA"}
!8 = !{!9, !9, i64 0}
!9 = !{!"long", !6, i64 0}
!10 = !{!6, !6, i64 0}
