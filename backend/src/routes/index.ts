import { Router } from 'express';
import livestreamRoutes from './livestream.routes.js';

const router = Router();

router.use('/livestream', livestreamRoutes);

export default router;
