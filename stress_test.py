import sys
import os
import threading
import time
import random

# Add bindings directory to path
sys.path.append(os.path.join(os.getcwd(), "telco_core/bindings"))

from telco_core import TelcoSimulator, TelcoError, QuotaType, QuotaBucket

def stress_test():
    db_path = os.path.join(os.getcwd(), "telco_core/simulator.db")
    # Clean up old DB for clean test
    if os.path.exists(db_path):
        os.remove(db_path)
        
    sim = TelcoSimulator("user_123", db_path)
    
    # Buy a YouTube topping (Video)
    video_topping = QuotaBucket(
        name="YouTube 2GB",
        remaining_bytes=2 * 1024 * 1024 * 1024,
        category=QuotaType.VIDEO,
        expiry=int(time.time()) + 3600
    )
    sim.buy_topping(video_topping)
    
    stats = {
        "success": 0,
        "insufficient_balance": 0,
        "total_calls": 0,
    }
    stats_lock = threading.Lock()

    def update_stats(kind):
        with stats_lock:
            stats["total_calls"] += 1
            stats[kind] += 1

    def user_thread(id):
        for _ in range(50):
            # 50% chance to use video data, 50% general
            cat = QuotaType.VIDEO if random.random() < 0.5 else QuotaType.GENERAL
            usage = random.randint(10 * 1024 * 1024, 100 * 1024 * 1024) # 10MB to 100MB
            try:
                sim.simulate_usage(usage, cat)
                update_stats("success")
            except TelcoError.InsufficientBalance:
                update_stats("insufficient_balance")
            time.sleep(0.005)

    threads = [threading.Thread(target=user_thread, args=(i,)) for i in range(10)]
    
    print("Starting Multi-Quota Persistence test...")
    for t in threads: t.start()
    for t in threads: t.join()
    
    print("\n--- Final State from DB ---")
    info = sim.get_account_info()
    for b in info.buckets:
        print(f"Bucket: {b.name} | Category: {b.category} | Remaining: {b.remaining_bytes / (1024*1024):.2f} MB")

    # Verify persistence by creating a new simulator instance pointing to same DB
    print("\n--- Testing Persistence (New Instance) ---")
    sim2 = TelcoSimulator("user_123", db_path)
    info2 = sim2.get_account_info()
    print(f"Buckets found in new instance: {len(info2.buckets)}")
    assert len(info2.buckets) == len(info.buckets)
    print("Persistence Verified!")

if __name__ == "__main__":
    stress_test()
