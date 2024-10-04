use crate::core_opt::core::*;
use crate::obj::mvcc::*;

pub mod core_opt;
pub mod obj;


fn main() {
    let eng = KVEngine::new();
    let mvcc = MVCC::new(eng);
    let tx0 = mvcc.begin_transaction();
    tx0.set(b"a", b"a1".to_vec());
    tx0.set(b"b", b"b1".to_vec());
    tx0.set(b"c", b"c1".to_vec());
    tx0.set(b"d", b"d1".to_vec());
    tx0.set(b"e", b"e1".to_vec());
    tx0.commit();

     // 开启一个事务
     let tx1 = mvcc.begin_transaction();
     // 将 a 改为 a2，e 改为 e2
     tx1.set(b"a", b"a2".to_vec());
     tx1.set(b"e", b"e2".to_vec());


     // Time
     //  1  a2              e2
     //  0  a1  b1  c1  d1  e1
     //     a   b   c   d   e   Keys
 
     // t1 虽然未提交，但是能看到自己的修改了
     tx1.print_all(); // a=a2 b=b1 c=c1 d=d1 e=e2


     // 开启一个新的事务
    let tx2 = mvcc.begin_transaction();
    // 删除 b
    tx2.delete(b"b");
    // Time
    //  2      X
    //  1  a2              e2
    //  0  a1  b1  c1  d1  e1
    //     a   b   c   d   e   Keys

    // 此时 T1 没提交，所以 T2 看到的是
    tx2.print_all(); // a=a1 c=c1 d=d1 e=e1
                     // 提交 T1
    tx1.commit();
    // 此时 T2 仍然看不到 T1 的提交，因为 T2 开启的时候，T2 还没有提交（可重复读）
    tx2.print_all(); // a=a1 c=c1 d=d1 e=e1


    // 再开启一个新的事务
    let tx3 = mvcc.begin_transaction();
    // Time
    //  3
    //  2      X               uncommitted
    //  1  a2              e2  committed
    //  0  a1  b1  c1  d1  e1
    //     a   b   c   d   e   Keys
    // T3 能看到 T1 的提交，但是看不到 T2 的提交
    tx3.print_all(); // a=a2 b=b1 c=c1 d=d1 e=e2

    // T3 写新的数据
    tx3.set(b"f", b"f1".to_vec());
    // T2 写同样的数据，会冲突
    tx2.set(b"f", b"f1".to_vec());
}
