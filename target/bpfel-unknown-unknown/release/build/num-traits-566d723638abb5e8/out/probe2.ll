; ModuleID = 'probe2.3a1fbbbh-cgu.0'
source_filename = "probe2.3a1fbbbh-cgu.0"
target datalayout = "e-m:e-p:64:64-i64:64-n32:64-S128"
target triple = "bpf"

; probe2::probe
; Function Attrs: norecurse nounwind readnone
define void @_ZN6probe25probe17h7140347c9b94c32aE() unnamed_addr #0 {
start:
  ret void
}

attributes #0 = { norecurse nounwind readnone "target-cpu"="generic" }

!llvm.module.flags = !{!0}

!0 = !{i32 7, !"PIC Level", i32 2}
