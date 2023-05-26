package org.rob.dev.simulator;

import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.util.Random;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

public class Simulator {
    private static final String ENGINE_HOST = "localhost";
    private static final int PORT = 8080;
    private final ScheduledExecutorService executorService;
    private final Random random;

    public Simulator() {
        this.executorService = Executors.newScheduledThreadPool(10);
        this.random = new Random();
    }

    public void run() {
        executorService.scheduleAtFixedRate(() -> {
            try (Socket socket = new Socket(ENGINE_HOST, PORT)) {
                var outputStream = socket.getOutputStream();

                for (int i = 0; i < 100; i++) {
                    char side = random.nextInt() % 2 == 0 ? 'B' : 'S';
                    int qty = random.nextInt(1, 5);
                    int px = random.nextInt(1, 5);

                    String order = String.format("%s,%d,%d\r\n", side, qty, px);
                    outputStream.write(order.getBytes(StandardCharsets.UTF_8));

                    System.out.printf("Sent at %d%n", System.currentTimeMillis());
                }

            } catch (Exception ex) {
                System.out.println("Error establishing connection: " + ex.getMessage());
            }
        }, 0, 50, TimeUnit.MICROSECONDS);
    }
}
